{ libfoo }:

libfoo.overrideAttrs (old: {
  # canibump:libfoo-crash
  patches = (old.patches or [ ]) ++ [ ./fix-crash-on-empty-input.patch ];
})
