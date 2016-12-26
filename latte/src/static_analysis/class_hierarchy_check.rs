use std::collections::HashMap;

use ast::{Class, Def, Ident, Program};

pub fn run(p: &Program) -> Result<(), String> {
    let mut classes: Vec<&Class> = Vec::new();
    for d in &p.0 {
        if let Def::DClass(ref c) = *d {
            classes.push(c);
        }
    }
    check_class_hierarchy(&classes)
}

fn check_class_hierarchy(classes_vec: &Vec<&Class>) -> Result<(), String> {
    let classes = get_classes(classes_vec)?;
    check_superclasses(&classes)?;
    check_hierarchy(&classes)?;
    Ok(())
}

fn get_classes<'a>(classes_vec: &Vec<&'a Class>) -> Result<HashMap<Ident, &'a Class>, String> {
    let mut classes: HashMap<Ident, &Class> = HashMap::new();
    for c in classes_vec {
        if classes.contains_key(&c.name) {
            return Err(format!("Multiple classes with name: {}", c.name));
        }
        classes.insert(c.name.clone(), c);
    }
    Ok(classes)
}

fn check_superclasses(classes: &HashMap<Ident, &Class>) -> Result<(), String> {
    for ref c in classes.values() {
        if let Some(ref superclass) = c.superclass {
            if !classes.contains_key(superclass) {
                return Err(format!("Cannot inherit from {}: undefined identifier", superclass));
            }
        }
    }
    Ok(())
}

fn check_hierarchy(classes: &HashMap<Ident, &Class>) -> Result<(), String> {
    let mut visited: HashMap<Ident, usize> = HashMap::new();
    for (index, ref c) in classes.values().enumerate() {
        visit(c, index, &mut visited, classes)?;
    }
    Ok(())
}

fn visit(class: &Class,
         step: usize,
         mut visited: &mut HashMap<Ident, usize>,
         classes: &HashMap<Ident, &Class>)
         -> Result<(), String> {
    if let Some(prev_step) = visited.get(&class.name) {
        if step == *prev_step {
            return Err(format!("Cycle in the class hierarchy"));
        }
        return Ok(());
    }

    visited.insert(class.name.clone(), step);
    if let Some(ref superclass_ident) = class.superclass {
        let superclass = classes.get(superclass_ident).unwrap();
        visit(superclass, step, &mut visited, classes)
    } else {
        Ok(())
    }
}
