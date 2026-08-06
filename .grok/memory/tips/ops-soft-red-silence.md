# Setup Tip — ops.soft-red-silence

- **event:** `smoke`
- **summary:** hermes loop smoke

## Rule

When `ops.soft-red-silence` appears, apply the control in `.grok/harness/failure-classes.v1.json` before expanding product scope.

## Do not

- Silent-drop soft reds.
- Call omc/omx/gjc/hermes CLIs as control plane.
