{ pkgs, ... }:

{
  environment.systemPackages = [
    # canibump:libfoo-crash the patched build, until upstream ships the fix
    (pkgs.callPackage ../pkgs/libfoo.nix { })
  ];
}
