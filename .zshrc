export EDITOR='emacsclient -cnw'

# The next line updates PATH for the Google Cloud SDK.
source '/Users/kofron/google-cloud-sdk/path.zsh.inc'

# The next line enables shell command completion for gcloud.
source '/Users/kofron/google-cloud-sdk/completion.zsh.inc'
#
# User configuration sourced by interactive shells
#

# Change default zim location
export ZIM_HOME=${ZDOTDIR:-${HOME}}/.zim

# Start zim
[[ -s ${ZIM_HOME}/init.zsh ]] && source ${ZIM_HOME}/init.zsh
