{ pkgs, ... }:

{
  # canibump:libfoo-crash
  # https://github.com/libfoo/libfoo/pull/412
  environment.systemPackages = [ (pkgs.callPackage ../pkgs/libfoo.nix { }) ];

  # Pinned until https://github.com/libbar/libbar/issues/77 is fixed.
  services.libbar.package = pkgs.libbar_2_1;

  # canibump:old-mesa-pin
  hardware.graphics.enable = true;

  # Background on the driver: https://gitlab.com/libbaz/libbaz/-/merge_requests/5 canibump-ignore
}
