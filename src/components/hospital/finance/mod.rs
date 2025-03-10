use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod invoice;
pub mod update;
pub mod view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinanceState {
    Invoice,
    View,
    Update,
}

pub struct Finance {
    pub state: FinanceState,
    pub invoice: invoice::InvoiceComponent,
    pub view_invoices: view::ViewInvoices,
    pub update_invoice: update::UpdateInvoice,
}

impl Finance {
    pub fn new() -> Self {
        let mut invoice = invoice::InvoiceComponent::new();
        invoice.load_patients().expect("Failed to load patients");
        let mut view_invoices = view::ViewInvoices::new();
        view_invoices
            .fetch_invoices()
            .expect("Failed to fetch invoices");
        let mut update_invoice = update::UpdateInvoice::new();
        update_invoice
            .fetch_invoices()
            .expect("Failed to fetch invoices");
        Self {
            state: FinanceState::Invoice,
            invoice,
            view_invoices,
            update_invoice,
        }
    }

    pub fn set_finance_state(&mut self, state: FinanceState) {
        self.state = state;
        match state {
            FinanceState::Invoice => {
                if let Err(e) = self.invoice.load_patients() {
                    eprintln!("Error initializing patient list: {}", e);
                }
            }
            FinanceState::View => {
                if let Err(e) = self.view_invoices.fetch_invoices() {
                    eprintln!("Error initializing invoice list on view: {}", e);
                }
            }
            FinanceState::Update => {
                if let Err(e) = self.update_invoice.fetch_invoices() {
                    eprintln!("Error initializing invoice list on update: {}", e);
                }
            }
        }
    }
}

impl Component for Finance {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.state {
            FinanceState::Invoice => {
                if let Some(action) = self.invoice.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
            FinanceState::View => {
                if let Some(action) = self.view_invoices.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
            FinanceState::Update => {
                if let Some(action) = self.update_invoice.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        match self.state {
            FinanceState::Invoice => self.invoice.render(frame),
            FinanceState::View => self.view_invoices.render(frame),
            FinanceState::Update => self.update_invoice.render(frame),
        }
    }
}
