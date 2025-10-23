function op-env --description 'Load environment variables from 1Password template'
  set -l template_path
  if set -q argv[1]
    set template_path $argv[1]
    if not string match -q '/*' "$template_path"
      set template_path "$HOME/.config/op/$template_path"
    end
  else
    set template_path "$HOME/.config/op/env.fish.tmpl"
  end

  if not command -q op
    echo "op-env: 1Password CLI not found in PATH" >&2
    return 1
  end

  if not test -f "$template_path"
    echo "op-env: template '$template_path' not found" >&2
    return 1
  end

  if not op whoami >/dev/null 2>&1
    set -l account_raw
    if set -q OP_ACCOUNT
      set account_raw $OP_ACCOUNT
    else
      set account_raw (op account list --format json | jq -r '.[0] | (.shorthand // .url // .account_uuid // empty)')
    end
    if test -z "$account_raw"
      echo "op-env: unable to determine 1Password account; set OP_ACCOUNT or sign in manually" >&2
      return 1
    end

    echo "op-env: signing into 1Password account '$account_raw'" >&2
    eval (op signin --account "$account_raw" --output fish 2>/dev/null)
    if test $status -ne 0
      echo "op-env: sign-in failed; unlock 1Password and try again" >&2
      return 1
    end
  end

  set -l tmp (mktemp 2>/dev/null)
  if test -z "$tmp"
    echo "op-env: unable to create temporary file" >&2
    return 1
  end

  if not op inject --in-file "$template_path" --out-file "$tmp"
    set -l inject_status $status
    rm -f "$tmp"
    return $inject_status
  end

  source "$tmp"
  set -l source_status $status
  rm -f "$tmp"
  return $source_status
end
