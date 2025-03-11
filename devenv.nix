{ ... }:
{
  languages.rust.enable = true;
  enterShell = ''
    export PATH="$PATH:${./unipac-shared}"
  '';
}
