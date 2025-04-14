{ ... }:
{
  languages.rust.enable = true;
  enterShell = ''
    export PATH="$PATH:$PWD/unipac-shared"
  '';
}
