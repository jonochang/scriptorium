{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "scriptorium-cli";
  version = "0.3.0";
  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Church bookstore platform";
    license = licenses.mit;
    platforms = platforms.unix;
  };
}
