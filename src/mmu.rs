//! Memory Management Unit (MMU) del Game Boy.
//!
//! La MMU è responsabile dell'accesso alla memoria principale del sistema.
//! In questa implementazione iniziale è rappresentata come un array continuo di
//! 65.536 byte, cioè 64 KiB, accessibile tramite indirizzi a 16 bit.

/// Struttura della Memory Management Unit.
///
/// Contiene il blocco di memoria principale del sistema, organizzato come un
/// array di byte di dimensione 64 KiB.
pub struct Mmu {
    data: [u8; 65536],
}

impl Mmu {
    /// Crea una nuova memoria vuota inizializzata a zero.
    ///
    /// La MMU viene inizializzata con tutto il contenuto a `0x00`, simile a una
    /// memoria fresca senza nessuna ROM o RAM caricata.
    ///
    /// # Returns
    /// Una nuova istanza di `Mmu` pronta all'uso.
    pub fn new() -> Self {
        Mmu { data: [0; 65536] }
    }

    /// Legge un byte dalla memoria all'indirizzo specificato.
    ///
    /// # Arguments
    /// * `address` - Indirizzo a 16 bit della cella da leggere.
    ///
    /// # Returns
    /// Il valore `u8` presente nella posizione di memoria richiesta.
    pub fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    /// Scrive un byte nella memoria all'indirizzo specificato.
    ///
    /// # Arguments
    /// * `address` - Indirizzo a 16 bit della cella da scrivere.
    /// * `value` - Byte da memorizzare nella posizione indicata.
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }
}
