function nd --description 'Start nix development shell in the current directory'
  command nix develop --extra-experimental-features nix-command --extra-experimental-features flakes
end
