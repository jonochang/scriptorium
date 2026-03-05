{ pkgs ? import <nixpkgs> {} }:
{
  scriptorium = pkgs.callPackage ./package.nix { };
}
