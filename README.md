# CBR2CBZ
A rust-nix commandline tool to convert CBR files to CBZ

## Todo
- [x] Update the cbz function to create the file in the tmp dir
  - [x] After finished, copy the file over to correct dir
- [x] Add commandline inputs
  - [x] Take input file name
    - [x] Check that files exist
    - [x] Check that input is CBZ file
  - [x] Add wildcard support
    - [x] For multiple inputs, make sure to create a new tmp dir for each (use input filename)
  - [x] Add debugging tooltips 
  - [x] Add option to delete input file
  - [x] Add option to change output filename 
  - [ ] Clean up terminal output
