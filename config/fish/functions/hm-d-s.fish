function hm-d-s --description 'Run home-manager switch for the kofron flake'
  command sudo nix run nix-darwin --extra-experimental-features nix-command --extra-experimental-features flakes -- switch --flake "$HOME/dotfiles/nix-darwin" $argv
end
