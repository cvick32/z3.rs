# Running BMC Test

`cargo run -- --filename examples/array_copy.vmt --bmc-count 0`

Currently `--bmc-count` doesn't do anything, so this will just run BMC on the input system up to a depth of 10. 

# Things we need (in no particular order)

- computing interpolants
- finding axiom violations from egg
- ranking egg rewrites
  - always prefer read terms?
  - only apply rewrites that fall into a particular vocabulary
- egraph from formula
  - getting the rewrites it induces
- rewrites from model + axiom instansitions 
