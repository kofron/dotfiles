{ config, pkgs, ...}:

{
  programs.git = {
    enable = true;
    userName = "Jared Kofron";
    userEmail = "jared@kofron.io";
  };
}
