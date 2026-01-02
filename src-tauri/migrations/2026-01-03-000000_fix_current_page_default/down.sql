-- Revert: set current_page back to 1 for unread books at page 0
UPDATE books SET current_page = 1 WHERE current_page = 0 AND reading_status = 'unread';
