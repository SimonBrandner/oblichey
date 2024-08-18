{
  self,
  system,
}: {
  lib,
  config,
  pkgs,
  ...
}: let
  cfg = config.programs.oblichey;
in
  with lib; {
    options.programs.oblichey = {
      enable = mkEnableOption "oblichey";
      package = mkOption {
        type = types.package;
        default = self.packages.${system}.default;
        description = ''
          The Oblichey package to use.
        '';
      };
      settings = {
        camera = {
          path = mkOption {
            type = types.str;
            description = "Path to the IR camera to be used.";
          };
        };
      };
      pamServices = mkOption {
        type = types.listOf types.str;
        description = "List of PAM services in which a rule for Oblichey should be added.";
      };
    };
    config = mkIf cfg.enable {
      environment = {
        systemPackages = [cfg.package];
        etc."oblichey.toml".text = ''
          [camera]
          path="${cfg.settings.camera.path}"
        '';
      };
      security.pam.services = lib.genAttrs cfg.pamServices (service: {
        rules.auth = {
          oblichey = {
            control = "sufficient";
            modulePath = "${self.packages.${system}.default}/lib/libpam_oblichey.so";
            order = config.security.pam.services.${service}.rules.auth.unix.order - 10;
          };
        };
      });
    };
  }
