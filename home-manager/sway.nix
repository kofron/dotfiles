{ config, pkgs, lib, ... }:

let
  modifier = "Mod4";
  terminal = "ghostty";
  launcher = "rofi -show drun";
  screenshotDir = "${config.home.homeDirectory}/Pictures/Screenshots";
  colors = {
    base = "#1e1e2e";
    mantle = "#181825";
    crust = "#11111b";
    text = "#cdd6f4";
    subtext = "#a6adc8";
    accent = "#89b4fa";
    critical = "#f38ba8";
    warning = "#f9e2af";
    success = "#a6e3a1";
  };
in
{
  wayland.windowManager.sway = {
    enable = true;
    package = pkgs.sway;
    systemd.enable = true;
    checkConfig = true;

    config = {
      inherit modifier terminal;
      bars = [ ];

      fonts = {
        names = [ "IosevkaTerm Nerd Font" "Noto Sans" "FiraCode Nerd Font" ];
        size = 13.0;
      };

      gaps = {
        inner = 1;
        outer = 1;
      };

      floating = {
        border = 2;
        criteria = [
          { app_id = "pavucontrol"; }
          { app_id = "blueman-manager"; }
          { title = "Picture-in-Picture"; }
        ];
      };

      focus = {
        followMouse = "always";
        newWindow = "smart";
      };

      input = {
        "type:touchpad" = {
          tap = "enabled";
          tap_button_map = "lrm";
          natural_scroll = "enabled";
          scroll_method = "two_finger";
          middle_emulation = "enabled";
          dwt = "enabled";
          click_method = "clickfinger";
          drag = "enabled";
          drag_lock = "enabled";
          accel_profile = "adaptive";
          pointer_accel = "0.35";
        };
      };

      output = lib.mkOptionDefault {
        "*" = {
          bg = "${colors.base} solid_color";
        };
        "eDP-1" = {
          scale = lib.mkForce "1";
        };
      };

      keybindings = lib.mkOptionDefault ({
        "${modifier}+Return" = "exec ${terminal}";
        "${modifier}+d" = "exec ${launcher}";
        "${modifier}+Shift+q" = "kill";
        "${modifier}+Shift+c" = "reload";
        "${modifier}+Shift+e" = "exec swaynag -t warning -m 'Exit Sway?' -b 'Logout' 'swaymsg exit'";
        "${modifier}+space" = lib.mkForce "floating toggle";
        "${modifier}+f" = "fullscreen toggle";
        "${modifier}+Shift+space" = lib.mkForce "focus mode_toggle";

        "${modifier}+Left" = "focus left";
        "${modifier}+Down" = "focus down";
        "${modifier}+Up" = "focus up";
        "${modifier}+Right" = "focus right";

        "${modifier}+Shift+Left" = "move left";
        "${modifier}+Shift+Down" = "move down";
        "${modifier}+Shift+Up" = "move up";
        "${modifier}+Shift+Right" = "move right";

        "${modifier}+Control+Left" = "resize shrink width 10px";
        "${modifier}+Control+Right" = "resize grow width 10px";
        "${modifier}+Control+Up" = "resize grow height 10px";
        "${modifier}+Control+Down" = "resize shrink height 10px";

        "${modifier}+Tab" = "workspace back_and_forth";
        "${modifier}+Shift+Tab" = "move container to workspace back_and_forth";

        "${modifier}+Print" = "exec grim - | wl-copy";
        "Print" = "exec mkdir -p ${screenshotDir} && grim ${screenshotDir}/$(date +'%Y-%m-%d_%H-%M-%S').png";
        "Shift+Print" = "exec mkdir -p ${screenshotDir} && grim -g $(slurp) ${screenshotDir}/$(date +'%Y-%m-%d_%H-%M-%S').png";

        "XF86AudioRaiseVolume" = "exec wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+";
        "XF86AudioLowerVolume" = "exec wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-";
        "XF86AudioMute" = "exec wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle";
        "XF86AudioMicMute" = "exec wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle";
        "XF86MonBrightnessUp" = "exec brightnessctl set +5%";
        "XF86MonBrightnessDown" = "exec brightnessctl set 5%-";
      }
        // builtins.listToAttrs (builtins.concatMap (w: [
          { name = "${modifier}+${builtins.toString w}"; value = "workspace number ${builtins.toString w}"; }
          { name = "${modifier}+Shift+${builtins.toString w}"; value = "move container to workspace number ${builtins.toString w}"; }
        ]) [1 2 3 4 5 6 7 8 9])
      );

      modes = {
        resize = {
          Escape = "mode default";
          Return = "mode default";
          Left = "resize shrink width 10px";
          Down = "resize grow height 10px";
          Up = "resize shrink height 10px";
          Right = "resize grow width 10px";
        };
      };

      startup = [
        { command = "waybar"; always = true; }
        { command = "mako"; always = true; }
        { command = "nm-applet"; }
      ];

      colors = {
        focused = {
          border = colors.accent;
          background = colors.base;
          text = colors.text;
          indicator = colors.accent;
          childBorder = colors.base;
        };
        focusedInactive = {
          border = colors.mantle;
          background = colors.base;
          text = colors.subtext;
          indicator = colors.mantle;
          childBorder = colors.base;
        };
        unfocused = {
          border = colors.crust;
          background = colors.base;
          text = colors.subtext;
          indicator = colors.crust;
          childBorder = colors.base;
        };
        urgent = {
          border = colors.critical;
          background = colors.critical;
          text = colors.base;
          indicator = colors.critical;
          childBorder = colors.critical;
        };
        placeholder = {
          border = colors.base;
          background = colors.base;
          text = colors.text;
          indicator = colors.base;
          childBorder = colors.base;
        };
        background = colors.base;
      };
    };

    extraConfig = ''
      smart_gaps on
      smart_borders on
      default_border pixel 1
      default_floating_border pixel 1
      titlebar_padding 1 2
      workspace_auto_back_and_forth yes
      for_window [class="^.*"] title_format "%title"
    '';

    extraSessionCommands = ''
      export MOZ_ENABLE_WAYLAND=1
      export QT_QPA_PLATFORM=wayland
      export SDL_VIDEODRIVER=wayland
      export XDG_CURRENT_DESKTOP=sway
      export XDG_SESSION_DESKTOP=sway
    '';
  };

  services.mako.enable = true;

  services.swayidle = {
    enable = true;
    systemdTarget = "sway-session.target";
    timeouts = [
      { timeout = 300; command = "swaylock -f"; }
      { timeout = 600; command = "swaymsg 'output * dpms off'"; }
    ];
    events = [
      { event = "before-sleep"; command = "swaylock -f"; }
      { event = "lock"; command = "swaymsg 'output * dpms off'"; }
      { event = "unlock"; command = "swaymsg 'output * dpms on'"; }
    ];
  };

  programs.swaylock = {
    enable = true;
    package = pkgs.swaylock-effects;
    settings = {
      effect-blur = "7x5";
      ring-color = colors.accent;
      ring-wrong-color = colors.critical;
      ring-ver-color = colors.success;
      inside-color = colors.mantle;
      inside-wrong-color = colors.critical;
      inside-ver-color = colors.success;
      text-color = colors.text;
      indicator = true;
      show-failed-attempts = true;
    };
  };

  xdg.configFile."waybar/config" = {
    source = ./waybar/config.json;
  };

  xdg.configFile."waybar/style.css" = {
    text = ''
      /* -- Global rules -- */
      * {
        border: none;
        font-family: "IosevkaTerm Nerd Font", "JetBrainsMono Nerd Font", monospace;
        font-size: 14px;
        min-height: 22px;
      }

      window#waybar {
        background: rgba(34, 36, 54, 0.6);
      }

      window#waybar.hidden {
        opacity: 0.2;
      }

      /* - General rules for visible modules -- */
      #custom-archicon,
      #clock,
      #cpu,
      #memory,
      #disk,
      #temperature,
      #idle_inhibitor,
      #battery,
      #pulseaudio,
      #pulseaudio_slider,
      #network,
      #keyboard-state {
        color: #161320;
        margin-top: 2px;
        margin-bottom: 2px;
        padding-left: 6px;
        padding-right: 6px;
        transition: none;
      }

      /* Separation to the left */
      #custom-archicon,
      #cpu,
      #idle_inhibitor {
        margin-left: 3px;
        border-top-left-radius: 6px;
        border-bottom-left-radius: 6px;
      }

      /* Separation to the right */
      #clock,
      #temperature,
      #keyboard-state {
        margin-right: 3px;
        border-top-right-radius: 6px;
        border-bottom-right-radius: 6px;
      }

      /* -- Specific styles -- */

      /* Modules left */
      #custom-archicon {
        font-size: 14px;
        color: #89B4FA;
        background: #161320;
        padding-right: 10px;
      }

      #clock {
        background: #ABE9B3;
      }

      #cpu {
        background: #96CDFB;
      }

      #memory {
        background: #DDB6F2;
      }

      #disk {
        background: #F5C2E7;
      }

      #temperature {
        background: #F8BD96;
      }

      /* Modules center */
      #workspaces {
        background: rgba(0, 0, 0, 0.5);
        border-radius: 6px;
        margin: 2px 3px;
        padding: 0px 4px;
      }

      #workspaces button {
        color: #B5E8E0;
        background: transparent;
        padding: 2px 2px;
        transition: color 0.3s ease, text-shadow 0.3s ease, transform 0.3s ease;
      }

      #workspaces button.occupied {
        color: #A6E3A1;
      }

      #workspaces button.focused,
      #workspaces button.active {
        color: #89B4FA;
        text-shadow: 0 0 4px #ABE9B3;
      }

      #workspaces button:hover {
        color: #89B4FA;
      }

      /* Modules right */
      #taskbar {
        background: transparent;
        border-radius: 6px;
        padding: 0px 3px;
        margin: 2px 3px;
      }

      #taskbar button {
        padding: 0px 3px;
        margin: 0px 2px;
        border-radius: 4px;
        transition: background 0.3s ease;
      }

      #taskbar button.active {
        background: rgba(34, 36, 54, 0.5);
      }

      #taskbar button:hover {
        background: rgba(34, 36, 54, 0.5);
      }

      #idle_inhibitor {
        background: #B5E8E0;
        padding-right: 8px;
      }

      #battery {
        background: #F9E2AF;
        padding-right: 8px;
      }

      #battery.charging,
      #battery.plugged {
        background: #A6E3A1;
      }

      #battery.warning {
        background: #F9E2AF;
      }

      #battery.critical {
        background: #F38BA8;
      }

      #pulseaudio {
        color: #1A1826;
        background: #F5E0DC;
        padding-right: 6px;
      }

      #pulseaudio_slider {
        color: #1A1826;
        background: #E8A2AF;
        padding-right: 6px;
      }

      #network {
        background: #CBA6F7;
        padding-right: 8px;
      }

      #keyboard-state {
        background: #A6E3A1;
        padding-right: 8px;
      }

      /* === Optional animation === */
      @keyframes blink {
        to {
          background-color: #BF616A;
          color: #B5E8E0;
        }
      }
    '';
  };
}
