//! # Disjoint Set Union (DSU) Deterministik
//!
//! Modul ini menyediakan struktur data *Disjoint Set Union* (Union-Find) yang dimodifikasi
//! untuk kebutuhan partisi netlist rangkaian logika digital sesuai Bab 3.4.2 dan Bab 3.10.11 proposal.
//!
//! Berbeda dari DSU biasa yang semata-mata mengoptimalkan representatif root berdasarkan rank/tinggi pohon,
//! implementasi ini secara aktif melacak dan mempertahankan **elemen kanonis minimum** (`canonical_min`)
//! di setiap himpunan. Dengan demikian, himpunan terhubung selalu memiliki pengenal deterministik
//! yang identik tanpa memedulikan urutan segmen kawat yang digabungkan (KNF-01).

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Struktur data *Disjoint Set Union* dengan pelacakan representatif kanonis berbasis nilai minimum.
#[derive(Debug, Clone, Default)]
pub struct DisjointSet<T>
where
    T: Copy + Ord + Hash + Eq,
{
    parent: HashMap<T, T>,
    rank: HashMap<T, usize>,
    canonical_min: HashMap<T, T>,
}

impl<T> DisjointSet<T>
where
    T: Copy + Ord + Hash + Eq,
{
    /// Membuat instance DSU baru yang kosong.
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
            canonical_min: HashMap::new(),
        }
    }

    /// Mendaftarkan elemen baru sebagai himpunan tunggal terisolasi jika belum ada.
    pub fn insert(&mut self, item: T) {
        if let Entry::Vacant(e) = self.parent.entry(item) {
            e.insert(item);
            self.rank.insert(item, 0);
            self.canonical_min.insert(item, item);
        }
    }

    /// Mencari root pohon dari suatu elemen dengan optimasi *path compression*.
    pub fn find(&mut self, item: T) -> T {
        self.insert(item);
        let mut curr = item;
        let mut path = Vec::new();

        while let Some(&p) = self.parent.get(&curr) {
            if p == curr {
                break;
            }
            path.push(curr);
            curr = p;
        }

        // Path compression: arahkan seluruh simpul pada lintasan langsung ke root
        for node in path {
            self.parent.insert(node, curr);
        }

        curr
    }

    /// Mengambil representatif kanonis (elemen dengan nilai minimum) dari himpunan tempat `item` berada.
    pub fn canonical(&mut self, item: T) -> T {
        let root = self.find(item);
        self.canonical_min.get(&root).copied().unwrap_or(item)
    }

    /// Menggabungkan dua himpunan yang memuat `a` dan `b`.
    ///
    /// Mengembalikan `true` jika kedua elemen sebelumnya berada di himpunan yang berbeda
    /// dan berhasil digabungkan, atau `false` jika keduanya sudah berada di himpunan yang sama.
    pub fn union(&mut self, a: T, b: T) -> bool {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return false;
        }

        let rank_a = self.rank.get(&root_a).copied().unwrap_or(0);
        let rank_b = self.rank.get(&root_b).copied().unwrap_or(0);

        let min_a = self.canonical_min.get(&root_a).copied().unwrap_or(root_a);
        let min_b = self.canonical_min.get(&root_b).copied().unwrap_or(root_b);
        let combined_min = min_a.min(min_b);

        let new_root = match rank_a.cmp(&rank_b) {
            std::cmp::Ordering::Less => {
                self.parent.insert(root_a, root_b);
                root_b
            }
            std::cmp::Ordering::Greater => {
                self.parent.insert(root_b, root_a);
                root_a
            }
            std::cmp::Ordering::Equal => {
                self.parent.insert(root_b, root_a);
                self.rank.insert(root_a, rank_a + 1);
                root_a
            }
        };

        self.canonical_min.insert(new_root, combined_min);
        true
    }

    /// Memeriksa apakah elemen `a` dan `b` berada dalam satu himpunan terhubung yang sama.
    pub fn is_connected(&mut self, a: T, b: T) -> bool {
        self.find(a) == self.find(b)
    }

    /// Mengembalikan seluruh kelompok himpunan yang terpartisi.
    ///
    /// Hasil dipetakan dalam `BTreeMap` di mana kunci (*key*) adalah representatif kanonis minimum
    /// dan nilai (*value*) adalah vektor anggota elemen yang diurutkan secara deterministik.
    pub fn sets(&mut self) -> BTreeMap<T, Vec<T>> {
        let all_elements: Vec<T> = self.parent.keys().copied().collect();
        let mut grouped: BTreeMap<T, Vec<T>> = BTreeMap::new();

        for elem in all_elements {
            let canon = self.canonical(elem);
            grouped.entry(canon).or_default().push(elem);
        }

        for members in grouped.values_mut() {
            members.sort_unstable();
        }

        grouped
    }

    /// Jumlah total elemen unik yang terdaftar pada DSU.
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    /// Memeriksa apakah DSU kosong.
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsu_basic_operations() {
        let mut dsu = DisjointSet::new();

        dsu.insert(10);
        dsu.insert(20);
        dsu.insert(30);

        assert_eq!(dsu.len(), 3);
        assert!(!dsu.is_connected(10, 20));

        assert!(dsu.union(10, 20));
        assert!(dsu.is_connected(10, 20));
        assert!(!dsu.is_connected(10, 30));

        // Union kembali tidak mengubah apa-apa
        assert!(!dsu.union(10, 20));
    }

    #[test]
    fn test_dsu_canonical_root_is_minimum() {
        let mut dsu1 = DisjointSet::new();
        // Gabungkan 50, 20, 80 dalam urutan acak
        dsu1.union(50, 20);
        dsu1.union(20, 80);

        assert_eq!(dsu1.canonical(50), 20);
        assert_eq!(dsu1.canonical(20), 20);
        assert_eq!(dsu1.canonical(80), 20);

        let mut dsu2 = DisjointSet::new();
        // Gabungkan elemen yang sama dalam urutan terbalik
        dsu2.union(80, 20);
        dsu2.union(20, 50);

        assert_eq!(dsu2.canonical(50), 20);
        assert_eq!(dsu2.canonical(20), 20);
        assert_eq!(dsu2.canonical(80), 20);

        // Kedua DSU harus menghasilkan partisi sets() yang identik persis
        assert_eq!(dsu1.sets(), dsu2.sets());
    }

    #[test]
    fn test_dsu_sets_partitioning() {
        let mut dsu = DisjointSet::new();

        dsu.union(1, 2);
        dsu.union(2, 3);

        dsu.union(10, 20);

        dsu.insert(100);

        let sets = dsu.sets();
        assert_eq!(sets.len(), 3);

        assert_eq!(sets.get(&1), Some(&vec![1, 2, 3]));
        assert_eq!(sets.get(&10), Some(&vec![10, 20]));
        assert_eq!(sets.get(&100), Some(&vec![100]));
    }
}
