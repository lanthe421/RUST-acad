use std::collections::HashMap;

/// Coin nominal values allowed in the vending machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coin {
    One = 1,
    Two = 2,
    Five = 5,
    Ten = 10,
    Twenty = 20,
    Fifty = 50,
}

impl Coin {
    pub fn value(self) -> u32 {
        self as u32
    }
}

/// A product available in the vending machine.
#[derive(Debug, Clone)]
pub struct Product {
    pub name: String,
    pub price: u32, // price in coin units
}

impl Product {
    pub fn new(name: impl Into<String>, price: u32) -> Self {
        Self {
            name: name.into(),
            price,
        }
    }
}

/// Errors that can occur during a vending machine purchase.
#[derive(Debug, PartialEq)]
pub enum VendingError {
    /// The slot index is invalid or empty.
    ProductNotFound,
    /// Inserted amount is less than the product price.
    InsufficientFunds { inserted: u32, price: u32 },
    /// Machine cannot make change for the given amount.
    CannotMakeChange,
    /// Machine is at full capacity for that slot.
    SlotFull,
}

impl std::fmt::Display for VendingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProductNotFound => write!(f, "Product not found"),
            Self::InsufficientFunds { inserted, price } => {
                write!(f, "Insufficient funds: inserted {inserted}, price {price}")
            }
            Self::CannotMakeChange => write!(f, "Cannot make change"),
            Self::SlotFull => write!(f, "Slot is full"),
        }
    }
}

impl std::error::Error for VendingError {}

/// Result of a successful purchase.
#[derive(Debug)]
pub struct PurchaseResult {
    pub product: Product,
    pub change: Vec<Coin>,
}

/// A vending machine with limited capacity per slot.
pub struct VendingMachine {
    /// Slots: each slot holds up to `capacity` copies of a product.
    slots: Vec<(Product, usize)>,
    /// Coins available in the machine for giving change.
    change_coins: HashMap<Coin, usize>,
    /// Max products per slot.
    capacity: usize,
}

impl VendingMachine {
    const COIN_DENOMINATIONS: [Coin; 6] = [
        Coin::Fifty,
        Coin::Twenty,
        Coin::Ten,
        Coin::Five,
        Coin::Two,
        Coin::One,
    ];

    /// Create a new vending machine with the given per-slot capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: Vec::new(),
            change_coins: HashMap::new(),
            capacity,
        }
    }

    /// Stock a product into a new slot. Returns the slot index.
    pub fn add_slot(&mut self, product: Product, count: usize) -> Result<usize, VendingError> {
        if count > self.capacity {
            return Err(VendingError::SlotFull);
        }
        self.slots.push((product, count));
        Ok(self.slots.len() - 1)
    }

    /// Load coins into the machine for making change.
    pub fn load_change(&mut self, coin: Coin, count: usize) {
        *self.change_coins.entry(coin).or_insert(0) += count;
    }

    /// Insert coins and buy the product at `slot`.
    /// Returns the product and any change coins on success.
    pub fn purchase(
        &mut self,
        slot: usize,
        inserted: impl IntoIterator<Item = Coin>,
    ) -> Result<PurchaseResult, VendingError> {
        let (product, count) = self.slots.get(slot).ok_or(VendingError::ProductNotFound)?;

        if *count == 0 {
            return Err(VendingError::ProductNotFound);
        }

        let total_inserted: u32 = inserted.into_iter().map(Coin::value).sum();
        let price = product.price;

        if total_inserted < price {
            return Err(VendingError::InsufficientFunds {
                inserted: total_inserted,
                price,
            });
        }

        let change_amount = total_inserted - price;
        let change = self.make_change(change_amount)?;

        // Commit: deduct product and change coins
        self.slots[slot].1 -= 1;
        for coin in &change {
            *self.change_coins.get_mut(coin).unwrap() -= 1;
        }

        Ok(PurchaseResult {
            product: self.slots[slot].0.clone(),
            change,
        })
    }

    /// Try to make change for `amount` using available coins (greedy).
    fn make_change(&self, mut amount: u32) -> Result<Vec<Coin>, VendingError> {
        if amount == 0 {
            return Ok(vec![]);
        }

        let mut result = Vec::new();
        let mut available = self.change_coins.clone();

        for &coin in &Self::COIN_DENOMINATIONS {
            let v = coin.value();
            let max_usable = available.get(&coin).copied().unwrap_or(0);
            let needed = (amount / v).min(max_usable as u32) as usize;
            for _ in 0..needed {
                result.push(coin);
                amount -= v;
                *available.get_mut(&coin).unwrap() -= 1;
            }
            if amount == 0 {
                break;
            }
        }

        if amount == 0 {
            Ok(result)
        } else {
            Err(VendingError::CannotMakeChange)
        }
    }

    /// View available products (slot index, name, price, remaining count).
    pub fn inventory(&self) -> impl Iterator<Item = (usize, &str, u32, usize)> {
        self.slots
            .iter()
            .enumerate()
            .map(|(i, (p, count))| (i, p.name.as_str(), p.price, *count))
    }
}

fn main() {
    let mut machine = VendingMachine::new(10);

    // Load change coins
    machine.load_change(Coin::Ten, 5);
    machine.load_change(Coin::Five, 5);
    machine.load_change(Coin::Two, 10);
    machine.load_change(Coin::One, 20);

    // Add products
    let cola_slot = machine
        .add_slot(Product::new("Cola", 35), 5)
        .expect("slot added");
    let chips_slot = machine
        .add_slot(Product::new("Chips", 20), 3)
        .expect("slot added");

    println!("=== Inventory ===");
    for (i, name, price, count) in machine.inventory() {
        println!("  [{i}] {name} — {price} coins ({count} left)");
    }

    // Buy Cola with exact change
    println!("\nBuying Cola with exact change (35 coins)...");
    match machine.purchase(
        cola_slot,
        [Coin::Twenty, Coin::Ten, Coin::Five],
    ) {
        Ok(r) => println!(
            "  Got: {}. Change: {:?}",
            r.product.name, r.change
        ),
        Err(e) => println!("  Error: {e}"),
    }

    // Buy Chips with overpayment
    println!("\nBuying Chips with 50-coin overpayment...");
    match machine.purchase(chips_slot, [Coin::Fifty]) {
        Ok(r) => println!(
            "  Got: {}. Change: {:?}",
            r.product.name, r.change
        ),
        Err(e) => println!("  Error: {e}"),
    }

    // Insufficient funds
    println!("\nTrying to buy Cola with only 10 coins...");
    match machine.purchase(cola_slot, [Coin::Ten]) {
        Ok(r) => println!("  Got: {}", r.product.name),
        Err(e) => println!("  Error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine_with_change() -> VendingMachine {
        let mut m = VendingMachine::new(10);
        m.load_change(Coin::Ten, 5);
        m.load_change(Coin::Five, 5);
        m.load_change(Coin::Two, 10);
        m.load_change(Coin::One, 20);
        m
    }

    #[test]
    fn exact_payment_no_change() {
        let mut m = machine_with_change();
        let slot = m.add_slot(Product::new("Water", 10), 1).unwrap();
        let r = m.purchase(slot, [Coin::Ten]).unwrap();
        assert_eq!(r.product.name, "Water");
        assert!(r.change.is_empty());
    }

    #[test]
    fn overpayment_gives_change() {
        let mut m = machine_with_change();
        let slot = m.add_slot(Product::new("Juice", 15), 1).unwrap();
        let r = m.purchase(slot, [Coin::Twenty]).unwrap();
        let change_total: u32 = r.change.iter().map(|c| c.value()).sum();
        assert_eq!(change_total, 5);
    }

    #[test]
    fn insufficient_funds_rejected() {
        let mut m = machine_with_change();
        let slot = m.add_slot(Product::new("Soda", 50), 1).unwrap();
        let err = m.purchase(slot, [Coin::Ten]).unwrap_err();
        assert_eq!(
            err,
            VendingError::InsufficientFunds {
                inserted: 10,
                price: 50
            }
        );
    }

    #[test]
    fn cannot_make_change_rejected() {
        let mut m = VendingMachine::new(10); // no change loaded
        let slot = m.add_slot(Product::new("Snack", 5), 1).unwrap();
        let err = m.purchase(slot, [Coin::Ten]).unwrap_err();
        assert_eq!(err, VendingError::CannotMakeChange);
    }

    #[test]
    fn empty_slot_returns_not_found() {
        let mut m = machine_with_change();
        let slot = m.add_slot(Product::new("Bar", 10), 0).unwrap();
        let err = m.purchase(slot, [Coin::Ten]).unwrap_err();
        assert_eq!(err, VendingError::ProductNotFound);
    }

    #[test]
    fn invalid_slot_returns_not_found() {
        let mut m = machine_with_change();
        let err = m.purchase(99, [Coin::Ten]).unwrap_err();
        assert_eq!(err, VendingError::ProductNotFound);
    }

    #[test]
    fn stock_decreases_after_purchase() {
        let mut m = machine_with_change();
        let slot = m.add_slot(Product::new("Tea", 5), 3).unwrap();
        m.purchase(slot, [Coin::Five]).unwrap();
        let (_, _, _, count) = m.inventory().find(|(i, ..)| *i == slot).unwrap();
        assert_eq!(count, 2);
    }
}
