# Iterator Patterns

## iter-filter-map

filter_map for combined operations.

```rust
// Parse strings to numbers, skipping invalid
let numbers: Vec<i32> = strings
    .iter()
    .filter_map(|s| s.parse().ok())
    .collect();
```

## iter-fold-accumulate

fold for reducing to single value.

```rust
let sum = numbers.iter().fold(0, |acc, x| acc + x);

// Group by category
let groups = items.iter().fold(
    HashMap::new(),
    |mut map, item| {
        map.entry(item.category())
            .or_insert_with(Vec::new)
            .push(item);
        map
    }
);
```

## iter-collect-turbofish

Turbofish for explicit types.

```rust
// Explicit collection type
let vec: Vec<i32> = iterator.collect::<Vec<_>>();

// Or with HashSet
let set: HashSet<i32> = iterator.collect::<HashSet<_>>();
```

## iter-custom-iterator

Custom Iterator implementation.

```rust
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < 6 {
            Some(self.count)
        } else {
            None
        }
    }
}
```

## iter-intoiterator

IntoIterator for for loops.

```rust
struct Grid {
    cells: Vec<Vec<i32>>,
}

impl IntoIterator for Grid {
    type Item = i32;
    type IntoIter = std::vec::IntoIter<i32>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter().flatten().collect::<Vec<_>>().into_iter()
    }
}

// Now works with for loop
let grid = Grid { cells: vec![vec![1, 2], vec![3, 4]] };
for cell in grid {
    println!("{}", cell);
}
```
