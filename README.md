# Running BMC Test

`cargo run -- --filename examples/array_copy.vmt --bmc-count 0`

Currently `--bmc-count` doesn't do anything, so this will just run BMC on the input system up to a depth of 10. 

# Notes
Currently (11/22), when you run `array_copy.vmt` you will find the following instances: 

```
[
  "(= (Read-Int-Int (Write-Int-Int b i 4) i) 4)",
  "(= (Read-Int-Int (Write-Int-Int b i 4) i) (Read-Int-Int b i))",
  "(= (Read-Int-Int (Write-Int-Int b i 5) i) 5)",
  "(= (Read-Int-Int (Write-Int-Int b i 5) i) (Read-Int-Int b i))",
  "(= (Read-Int-Int (Write-Int-Int b i 3) i_next) (Read-Int-Int b i_next))",
]
```

I don't like these instances as they contain subterms that are non-variable, 4 and 5. 
For more difficult problems, these instances won't generalize as they refer directly
to a particular case, the case where 4 is written into array b for the first instance. 

To get around this we need to be smarter about how we choose substitutions in the 
conflict scheduler. I thought something like the below would work but the type 
of `new_term_without_constants` is `RecExpr<L>` and not `L`. I couldn't figure
out a way to build a term that was of type `L` directly. 

But this problem seems like something egg should deal with directly, like we have a 
node in the egraph and we want to look at all the equivalent terms and choose the 
one we like the best. I think I'm just thinking about it the wrong way. 

```rust
for node in &eclass.nodes {
  if node.to_string().contains(VARIABLE_FRAME_DELIMITER) {
    // Always return a variable if one is available.
    return node.clone();
  } else if node.children().len() > 0 {
    let new_children = |id: Id| find_best_variable_substitution(egraph, &egraph[id]);
    let new_term_without_constants = node.build_recexpr(new_children);
    println!("{:?}", egraph.lookup_expr(&dd));
    println!("new term: {}", dd);
    return new_term_without_constants;
  }
}
```



# Things we need (in no particular order)

- computing interpolants
- finding axiom violations from egg
- ranking egg rewrites
  - always prefer read terms?
  - only apply rewrites that fall into a particular vocabulary
- egraph from formula
  - getting the rewrites it induces
- rewrites from model + axiom instansitions 
