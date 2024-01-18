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
        default = "depth";
        description = "The folder to save recordings to within the deployment directory.";
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
        # DEPLOYMENT_DIRECTORY is set by the deployment-start service
        OUTPUT_PATH=$DEPLOYMENT_DIRECTORY/${config.services.audio-recorder.output-folder}
        ${pkgs.coreutils}/bin/mkdir -p $OUTPUT_PATH
        RUST_LOG=info ${depthRecorder}/bin/depth-recorder \
        $OUTPUT_PATH \
        ${toString config.services.depth-recorder.interval-secs}
      '';
      serviceConfig = {
        User = "root";
        Restart = "always";
      };
      unitConfig = {
        After = ["multi-user.target" "deployment-start.service"];
      };
      startLimitIntervalSec = 0;
    };
  };
}
