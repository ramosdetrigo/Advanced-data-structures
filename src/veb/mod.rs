use std::collections::HashMap;

pub struct Veb {
    w: u32,
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
            let (c, i) = split_bits(x, self.w);

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

    pub fn remove(&mut self, mut x: u32) {
        if self.min.is_none() {
            // veb vazia
            return;
        }

        if x == self.min.unwrap() {
            // Não existe próximo cluster não-vazio: tudo vazio. É só remover o min
            if self.resumo.min.is_none() {
                self.min = None;
                return;
            }

            // x <- v.min <- <c, i <- v.cluster[c].min>
            let c_next = self.resumo.min.unwrap(); // min is_some
            let i_min = self.clusters[&c_next].min.unwrap(); // cluster não-vazio
            let merge = merge_bits(c_next, i_min, self.w);

            self.min = Some(merge);
            x = merge;
        }

        let (c, i) = split_bits(x, self.w);
        let cluster = self.clusters.get_mut(&c).unwrap();
        cluster.remove(i);

        if cluster.min.is_none() {
            // cluster c ficou vazio: remove ele do resumo
            self.resumo.remove(c);
        }
        if self.resumo.min.is_none() {
            // Não existe cluster não-vazio: max = min
            self.max = self.min
        } else {
            let c_m = self.resumo.max.unwrap(); // min is_some -> max is_some
            let i_m = self.clusters[&c_m].max.unwrap(); // cluster não-vazio
            self.max = Some(merge_bits(c_m, i_m, self.w));
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
        let (c, i) = split_bits(x, self.w);

        if let Some(cluster) = &self.clusters.get(&c) {
            if cluster.max.is_some_and(|max| i < max) {
                return Some(merge_bits(c, cluster.successor(x).unwrap(), self.w));
            }
        }

        // Caso 3: resposta não está no cluster c -> checa resumo
        let next_c = self.resumo.successor(c)?; // procura primeiro cluster não-vazio depois de C
        let next_i = self.clusters[&next_c].min.unwrap(); // unwrap garantido: cluster não vazio
        Some(merge_bits(next_c, next_i, self.w))
    }
}

fn split_bits(x: u32, w: u32) -> (u32, u32) {
    (higher_bits(x, w), lower_bits(x, w))
}

fn lower_bits(x: u32, w: u32) -> u32 {
    x & ((1 << w / 2) - 1)
}

fn higher_bits(x: u32, w: u32) -> u32 {
    x >> w / 2
}

fn merge_bits(c: u32, i: u32, w: u32) -> u32 {
    (c << w / 2) | i
}
