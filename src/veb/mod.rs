use std::collections::HashMap;

const W: u32 = 32;

pub struct Veb {
    u: u32,
    min: Option<u32>,
    max: Option<u32>, // CÓPIA do maior elemento em V
    // w' = w/2
    clusters: HashMap<u32, Box<Veb>>,
    resumo: Box<Veb>,
}

impl Veb {
    pub fn include(&mut self, mut x: u32) {
        if self.min.is_none() {
            self.min = Some(x);
            self.max = Some(x);
        } else {
            let c = higher_bits(x);
            let i = lower_bits(x);

            if x < self.min.unwrap() {
                std::mem::swap(&mut x, self.min.as_mut().unwrap());
            }

            if x > self.max.unwrap() {
                self.max = Some(x);
            }

            if self.clusters[&c].min.is_none() {
                self.resumo.include(c);
            }

            self.clusters.get_mut(&c).unwrap().include(i);
        }
    }

    pub fn successor(&self, x: u32) -> Option<u32> {
        // Caso 1: check básico de min/max
        if x < self.min? {
            return self.min;
        } else if x >= self.max? {
            return None;
        }

        // Caso 2: checa se resposta está no cluster c
        let i = lower_bits(x);
        let c = higher_bits(x);

        if let Some(cluster) = &self.clusters.get(&c) {
            if cluster.max.is_some_and(|max| i < max) {
                return Some(merge_bits(c, cluster.successor(x).unwrap()));
            }
        }

        // Caso 3: resposta não está no cluster c -> checa resumo
        let next_c = self.resumo.successor(c)?; // procura primeiro cluster não-vazio depois de C
        let next_i = self.clusters[&next_c].min.unwrap(); // unwrap garantido: cluster não vazio
        Some(merge_bits(next_c, next_i))
    }
}

fn lower_bits(x: u32) -> u32 {
    x & ((1 << W / 2) - 1)
}

fn higher_bits(x: u32) -> u32 {
    x >> W / 2
}

fn merge_bits(c: u32, i: u32) -> u32 {
    (c << W / 2) | i
}
