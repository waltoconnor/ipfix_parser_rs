use crate::templates::IPFIXTemplate;

use std::{collections::HashMap};

//ring here is used like "keyring"
pub struct TemplateRing {
    //(id, odid) -> IPFIXTemplate
    templates: HashMap<(u16, u32), IPFIXTemplate>,
}

impl TemplateRing {
    pub fn new() -> Self {
        TemplateRing { templates: HashMap::new() }
    }

    pub fn insert_template(&mut self, template: IPFIXTemplate, odid: u32) {
        match self.templates.insert((template.id, odid), template) {
            None => {},
            Some(_k) => { /*TODO: log replacement of old template*/ }
        };
    }

    pub fn get_template(&self, id: u16, odid: u32) -> Option<IPFIXTemplate> {
        match self.templates.get(&(id, odid)){
            None => None,
            Some(v) => Some(v.clone())
        }
    }

    // pub fn prune_old_templates(&mut self, max_age: Duration) {
    //     let dead_ids: Vec<u16> = self.last_used.lock().unwrap().iter()
    //         .filter(|(_id, time)| { time.elapsed() > max_age })
    //         .map(|(id, _time)| *id)
    //         .collect();

    //     for id in dead_ids {
    //         self.templates.remove(&id);
    //         self.last_used.lock().unwrap().remove(&id);
    //     }
    // }

}


