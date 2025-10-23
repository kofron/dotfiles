{ config, pkgs, ... }:

{
  imports = [
    ./sway.nix
    ./git.nix
  ];
  # Home Manager needs a bit of information about you and the paths it should
  # manage.
  home.username = "kofron";
  home.homeDirectory = "/home/kofron";

  # This value determines the Home Manager release that your configuration is
  # compatible with. This helps avoid breakage when a new Home Manager release
  # introduces backwards incompatible changes.
  #
  # You should not change this value, even if you update Home Manager. If you do
  # want to update the value, then make sure to first check the Home Manager
  # release notes.
  home.stateVersion = "25.05"; # Please read the comment before changing.

  nixpkgs = {
    config = {
      allowUnfree = true;
      allowUnfreePredicate = (_: true);
    };
  };

  # The home.packages option allows you to install Nix packages into your
  # environment.
  home.packages = [
    # # Adds the 'hello' command to your environment. It prints a friendly
    # # "Hello, world!" when run.
    # pkgs.hello

    # # It is sometimes useful to fine-tune packages, for example, by applying
    # # overrides. You can do that directly here, just don't forget the
    # # parentheses. Maybe you want to install Nerd Fonts with a limited number of
    # # fonts?
    # (pkgs.nerdfonts.override { fonts = [ "FantasqueSansMono" ]; })

    # # You can also create simple shell scripts directly inside your
    # # configuration. For example, this adds a command 'my-hello' to you
    # # environment:
    # (pkgs.writeShellScriptBin "my-hello" ''
    #   echo "Hello, ${config.home.username}!"
    # '')
    pkgs.waybar
    pkgs.grim
    pkgs.slurp
    pkgs.wl-clipboard
    pkgs.brightnessctl
    pkgs.nerd-fonts.iosevka-term
    pkgs.iosevka
    pkgs.keymapp
    pkgs.bun
    pkgs.nodejs_22
    pkgs.zellij
    pkgs.mkcert
    pkgs.jq
    pkgs.brave
    pkgs.signal-desktop
    pkgs.ripgrep
    pkgs.nixfmt
  ];

  # Home Manager is pretty good at managing dotfiles. The primary way to manage
  # plain files is through 'home.file'.
  home.file = {
    # # Building this configuration will create a copy of 'dotfiles/screenrc' in
    # # the Nix store. Activating the configuration will then make '~/.screenrc' a
    # # symlink to the Nix store copy.
    # ".screenrc".source = dotfiles/screenrc;

    # # You can also set the file content immediately.
    # ".gradle/gradle.properties".text = ''
    #   org.gradle.console=verbose
    #   org.gradle.daemon.idletimeout=3600000
    # '';
    ".config/zed/settings.json".text = ''
      {
        "edit_predictions": {
          "mode": "subtle",
          "copilot": {
            "proxy": null,
            "proxy_no_verify": null
          },
          "enabled_in_text_threads": false
        },
        "show_edit_predictions": true,
        "features": {
          "edit_prediction_provider": "zed"
        },
        "inlay_hints": {
          "toggle_on_modifiers_press": {
            "control": true,
            "shift": false,
            "alt": false,
            "platform": false,
            "function": false
          }
        },
        "theme": "Catppuccin Macchiato",
        "ui_font_family": "Iosevka",
        "buffer_font_family": "Iosevka",
        "ui_font_size": 8,
        "buffer_font_size": 8,
        "lsp": {

          "pyright": {
            "settings": {
              "python.analysis": {
                "diagnosticMode": "workspace",
                "typeCheckingMode": "strict"
              },
              "python": {
                "pythonPath": ".venv/bin/python"
              }
            }
          }
        },
        "languages": {
          "Markdown": {
            "show_edit_predictions": false
          },
          "SQL": {
            "language_servers": [
              "postgres_lsp"
            ],
            "enable_language_server": true
          },
          "Rust": {
            "format_on_save": "on",
            "formatter": "language_server"
          },
          "Python": {
            "language_servers": [
              "pyright",
              "ruff"
            ],
            "format_on_save": "on",
            "formatter": [
              {
                "code_actions": {
                  "source.organizeImports.ruff": true,
                  "source.fixAll.ruff": true
                }
              },
              {
                "language_server": {
                  "name": "ruff"
                }
              }
            ]
          }
        }
      }
    '';
    ".config/doom" = {
      source = ../config/doom;
      recursive = true;
    };
    ".config/fish/functions/e.fish".source = ../config/fish/functions/e.fish;
    ".config/fish/functions/hm-s.fish".source = ../config/fish/functions/hm-s.fish;
    ".config/fish/functions/op-env.fish".source = ../config/fish/functions/op-env.fish;
    ".config/op/env.fish.tmpl".source = ../config/op/env.fish.tmpl;
  };

  # Home Manager can also manage your environment variables through
  # 'home.sessionVariables'. These will be explicitly sourced when using a
  # shell provided by Home Manager. If you don't want to manage your shell
  # through Home Manager then you have to manually source 'hm-session-vars.sh'
  # located at either
  #
  #  ~/.nix-profile/etc/profile.d/hm-session-vars.sh
  #
  # or
  #
  #  ~/.local/state/nix/profiles/profile/etc/profile.d/hm-session-vars.sh
  #
  # or
  #
  #  /etc/profiles/per-user/kofron/etc/profile.d/hm-session-vars.sh
  #
  home.sessionVariables = {
    EDITOR = "emacs";
    GTK_USE_PORTAL = 0;
  };

  # Let Home Manager install and manage itself.
  programs.home-manager.enable = true;

  programs.rofi = {
    enable = true;
    package = pkgs.rofi-wayland;
    extraConfig = {
      modi = "drun,run,window";
      show-icons = true;
      font = "IosevkaTerm Nerd Font 16";
      drun-display-format = "{name}";
    };
  };

  programs.ghostty.enable = true;
  programs.ghostty.settings = {
    "font-family" = "IosevkaTerm Nerd Font";
    "font-size" = 12;
  };

  programs.fish = {
    enable = true;
    plugins = [
      {
        name = "pure";
        src = pkgs.fishPlugins.pure.src;
      }
    ];
    interactiveShellInit = ''
      set -g fish_greeting
    '';
  };
}
