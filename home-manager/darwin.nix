{ config, pkgs, pkgsUnstable, zedPkg, ... }:

{
  imports = [ ./git.nix ];
  # Home Manager needs a bit of information about you and the paths it should
  # manage.
  home.username = "kofron";
  home.homeDirectory = "/Users/kofron";

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
  # most packages from stable
  home.packages = with pkgs;
    [];

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
    ".config/zed/settings.json".source = ./zed/settings.json;
    ".config/zed/keymap.json".source = ./zed/keymap.json;
    ".config/doom" = {
      source = ../config/doom;
      recursive = true;
    };
    ".config/fish/functions/e.fish".source = ../config/fish/functions/e.fish;
    ".config/fish/functions/hm-s.fish".source =
      ../config/fish/functions/hm-s.fish;
      ".config/fish/functions/hm-d-s.fish".source =
        ../config/fish/functions/hm-d-s.fish;
    ".config/fish/functions/op-env.fish".source =
      ../config/fish/functions/op-env.fish;
    ".config/fish/functions/nd.fish".source = ../config/fish/functions/nd.fish;
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
  };

  # Let Home Manager install and manage itself.
  programs.home-manager.enable = true;

  programs.emacs = {
    enable = true;
    extraPackages = epkgs: [ epkgs.vterm ];
  };

  programs.fish = {
    enable = true;
    plugins = [{
      name = "pure";
      src = pkgs.fishPlugins.pure.src;
    }];
    interactiveShellInit = ''
      set -g fish_greeting
      # Source - https://stackoverflow.com/a
      # Posted by Maximilian, modified by community. See post 'Timeline' for change history
      # Retrieved 2025-12-09, License - CC BY-SA 4.0
      contains /path $fish_user_paths; or set -Ua fish_user_paths Users/kofron/.nix-profile/bin /etc/profiles/per-user/kofron/bin /run/current-system/sw/bin /nix/var/nix/profiles/default/bin

    '';
  };
}
