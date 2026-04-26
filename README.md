# CBR2CBZ
A rust-nix commandline tool to convert CBR files to CBZ

## Todo
- [ ] Update the cbz function to create the file in the tmp dir
  - [ ] After finished, copy the file over to correct dir
- [ ] Add commandline inputs
  - [ ] Take input file name
    - [ ] Check that files exist
    - [ ] Check that input is CBZ file
  - [ ] Add wildcard support
    - [ ] For multiple inputs, make sure to create a new tmp dir for each (use input filename)
  - [ ] Add debugging tooltips 
  - [ ] Add option to delete input file
  - [ ] Add option to change output filename 
