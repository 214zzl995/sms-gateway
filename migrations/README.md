# Database Migrations

## Schema Version 0.3

This migration creates the complete database structure for the SMS Gateway with the new decoupled architecture.

### Key Features

1. **Device-IMSI Decoupling**: SMS messages are now associated with SIM cards rather than device names
2. **SIM Card Management**: Central `sim_cards` table with phone numbers and user-defined aliases
3. **Alias System**: Automatic alias generation with fallback logic
4. **Enhanced Information**: Support for extended modem and SIM card data

### Database Structure

#### Tables
- `contacts`: Contact management
- `sms`: SMS message storage (with `sim_id` reference)
- `sim_cards`: SIM card information and metadata

#### Views
- `v_contacts`: Contact list with latest message information
- `v_sim_info`: SIM card view with computed effective aliases

### Migration Notes

- The `device` column has been completely removed from the `sms` table
- All SMS entries now use the `sim_id` field to reference SIM cards
- Complete decoupling from device names achieved
- Aliases are automatically generated using the priority: `alias > phone_number > SIM-{last4digits}`

### Fresh Installation

For fresh installations, this single migration file creates the complete database structure.
No previous migration files are needed.