-- Update any existing books that still have current_page = 1 and are unread
UPDATE books SET current_page = 0 WHERE current_page = 1 AND reading_status = 'unread';
