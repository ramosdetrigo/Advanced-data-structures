use super::Veb;
use std::ops::Index;

const INIT_CAP: usize = 16;

/// hashmap simples específico para chaves u32.
/// utiliza encadeamento com vetores para tratar colisões.
pub struct VEBHashMap {
    buckets: Vec<Vec<(u32, Veb)>>,
    len: usize,
}

impl VEBHashMap {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let mut buckets = Vec::with_capacity(INIT_CAP);
        for _ in 0..INIT_CAP {
            buckets.push(Vec::new());
        }
        Self { buckets, len: 0 }
    }

    /// Dobra a capacidade dos buckets
    fn double(&mut self) {
        // aloca nova tabela
        let new_capacity = self.buckets.len() * 2;
        let mut new_buckets = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            new_buckets.push(Vec::new());
        }

        // move os elementos da tabela antiga pra nova
        for bucket in self.buckets.drain(..) {
            for (k, v) in bucket {
                let idx = hash(k, new_capacity);
                new_buckets[idx].push((k, v));
            }
        }
        self.buckets = new_buckets;
    }

    fn halve(&mut self) {
        // aloca nova tabela
        let new_capacity = self.buckets.len() / 2;
        let mut new_buckets = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            new_buckets.push(Vec::new());
        }

        // move os elementos da tabela antiga pra nova
        for bucket in self.buckets.drain(..) {
            for (k, v) in bucket {
                let idx = hash(k, new_capacity);
                new_buckets[idx].push((k, v));
            }
        }
        self.buckets = new_buckets;
    }

    /// Insere chave-valor no hashmap
    pub fn insert(&mut self, key: u32, value: Veb) {
        // duplica o tamanho da tabela se mais de 3/4 da capacidade total foi usada
        if self.len >= self.buckets.len() * 3 / 4 {
            self.double();
        }

        let idx = hash(key, self.buckets.len());
        let bucket = &mut self.buckets[idx];

        // Atualiza se a chave já existir
        for &mut (k, ref mut v) in bucket.iter_mut() {
            if k == key {
                *v = value;
                return;
            }
        }

        bucket.push((key, value));
        self.len += 1;
    }

    /// Remove um elemento do hashmap, retornando-o. Retorna None se o elemento não está no hashmap.
    pub fn remove(&mut self, key: &u32) -> Option<Veb> {
        let capacity = self.buckets.len();
        let idx = hash(*key, capacity);
        let bucket = &mut self.buckets[idx];

        // busca no bucket
        if let Some(pos) = bucket.iter().position(|(k, _)| k == key) {
            // remove trocando com o último item do vec - O(1)
            let removed_val = bucket.swap_remove(pos).1;
            self.len -= 1;

            // table halving se consumo <= 1/4 da capacidade
            if self.len > 0 && self.len <= capacity / 4 {
                self.halve();
            }

            return Some(removed_val);
        }

        None
    }

    /// Retorna um Option com uma referência ao elemento na chave de entrada. None se não existe
    pub fn get(&self, key: &u32) -> Option<&Veb> {
        let idx = hash(*key, self.buckets.len());
        for (k, v) in &self.buckets[idx] {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    /// Retorna um Option com uma referência mutável ao elemento na chave de entrada. None se não existe
    pub fn get_mut(&mut self, key: &u32) -> Option<&mut Veb> {
        let idx = hash(*key, self.buckets.len());
        for (k, v) in &mut self.buckets[idx] {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    /// Retorna true se existe um elemento nessa chave no hashmap
    pub fn contains_key(&self, key: &u32) -> bool {
        self.get(key).is_some()
    }
}

/// Função de hash simples usando módulo
#[inline]
fn hash(key: u32, capacity: usize) -> usize {
    (key as usize) % capacity
}

// permite index com []
impl Index<u32> for VEBHashMap {
    type Output = Veb;
    fn index(&self, index: u32) -> &Self::Output {
        self.get(&index).unwrap()
    }
}
