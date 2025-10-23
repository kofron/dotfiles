function hm-s --description 'Run home-manager switch for the kofron flake'
  command home-manager switch --flake "$HOME/dotfiles#kofron@lifschitz" --extra-experimental-features "nix-command flakes" $argv
end
