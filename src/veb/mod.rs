use std::{cell::OnceCell, collections::HashMap};

pub struct Veb {
    w: u32,
    min: Option<u32>,
    max: Option<u32>, // CÓPIA do maior elemento em V
    // w' = w/2
    clusters: HashMap<u32, Veb>,
    resumo: OnceCell<Box<Veb>>,
}

impl Veb {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_w(32)
    }

    #[inline]
    #[must_use]
    fn with_w(w: u32) -> Self {
        Self {
            w,
            min: None,
            max: None,
            clusters: HashMap::new(),
            resumo: OnceCell::new(),
        }
    }

    #[inline]
    #[must_use]
    fn resumo(&self) -> &Veb {
        self.resumo
            .get_or_init(|| Box::new(Self::with_w(self.w / 2)))
    }

    #[inline]
    #[must_use]
    fn resumo_mut(&mut self) -> &mut Veb {
        self.resumo
            .get_or_init(|| Box::new(Self::with_w(self.w / 2)));
        self.resumo.get_mut().unwrap()
    }

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
                self.resumo_mut().include(c);
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
            if self.resumo().min.is_none() {
                self.min = None;
                return;
            }

            // x <- v.min <- <c, i <- v.cluster[c].min>
            let c_next = self.resumo().min.unwrap(); // min is_some
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
            self.resumo_mut().remove(c);
        }
        if self.resumo().min.is_none() {
            // Não existe cluster não-vazio: max = min
            self.max = self.min
        } else {
            let c_m = self.resumo().max.unwrap(); // min is_some -> max is_some
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

        if let Some(cluster) = self.clusters.get(&c) {
            if cluster.max.is_some_and(|max| i < max) {
                let succ_i = cluster.successor(i).unwrap();
                return Some(merge_bits(c, succ_i, self.w));
            }
        }

        // Caso 3: resposta não está no cluster c -> checa resumo
        let next_c = self.resumo().successor(c)?; // procura primeiro cluster não-vazio depois de C
        let next_i = self.clusters[&next_c].min.unwrap(); // unwrap garantido: cluster não vazio
        Some(merge_bits(next_c, next_i, self.w))
    }

    pub fn predecessor(&self, x: u32) -> Option<u32> {
        // Caso 1: check básico de min/max
        if x > self.max? {
            return self.max;
        } else if x <= self.min? {
            return None;
        }

        // Caso 2: checa se resposta está no cluster c
        let (c, i) = split_bits(x, self.w);

        if let Some(cluster) = self.clusters.get(&c) {
            if cluster.min.is_some_and(|min| i > min) {
                let pred_i = cluster.predecessor(i).unwrap();
                return Some(merge_bits(c, pred_i, self.w));
            }
        }

        // Caso 3: resposta não está no cluster c -> checa resumo
        let prev_c = self.resumo().predecessor(c)?; // procura primeiro cluster não-vazio depois de C
        let prev_i = self.clusters[&prev_c].max.unwrap(); // unwrap garantido: cluster não vazio
        Some(merge_bits(prev_c, prev_i, self.w))
    }
}

#[inline]
#[must_use]
fn split_bits(x: u32, w: u32) -> (u32, u32) {
    (higher_bits(x, w), lower_bits(x, w))
}

#[inline]
#[must_use]
fn lower_bits(x: u32, w: u32) -> u32 {
    x & ((1 << w / 2) - 1)
}

#[inline]
#[must_use]
fn higher_bits(x: u32, w: u32) -> u32 {
    x >> w / 2
}

#[inline]
#[must_use]
fn merge_bits(c: u32, i: u32, w: u32) -> u32 {
    (c << w / 2) | i
}
