function e --description 'Open a terminal-frame Emacs via emacsclient'
  command emacsclient -nw -s /run/user/1000/emacs/server $argv
end

function hm-s --description 'Run home-manager switch for the kofron flake'
  set -l flake "$HOME/dotfiles#kofron@lifschitz"
  command home-manager switch --flake $flake --extra-experimental-features "nix-command flakes" $argv
end
