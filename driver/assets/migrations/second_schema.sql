ALTER TABLE config 
ADD COLUMN email_locale INTEGER DEFAULT 0;
UPDATE config SET email_locale = 0 WHERE email_locale IS NULL;
