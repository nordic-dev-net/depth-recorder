{
  config,
  pkgs,
  lib,
  ...
}: let
  depthRecorder = pkgs.callPackage ./default.nix {};
in {
  options = {
    services.depth-recorder = {
      enable = lib.mkEnableOption "Whether to enable the depth-recorder service.";

      output-path = lib.mkOption {
        type = lib.types.str;
        default = "/output/depth";
        description = "The folder to save recordings to.";
      };

      interval-secs = lib.mkOption {
        type = lib.types.int;
        default = 10;
        description = "Interval of recording depth in seconds.";
      };
    };
  };

  config = lib.mkIf config.services.depth-recorder.enable {
    systemd.services.depth-recorder = {
      description = "Depth recorder service";
      wantedBy = ["multi-user.target"];
      script = ''
        #!/usr/bin/env bash
        set -x
        ${pkgs.coreutils}/bin/mkdir -p ${config.services.depth-recorder.output-path}
        RUST_LOG=info ${depthRecorder}/bin/depth-recorder \
        ${config.services.depth-recorder.output-path} \
        ${toString config.services.depth-recorder.interval-secs}
      '';
      serviceConfig = {
        User = "root";
        Restart = "always";
      };
      unitConfig = {
        After = "multi-user.target";
      };
      startLimitIntervalSec = 0;
    };
  };
}
