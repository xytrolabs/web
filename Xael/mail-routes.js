// ─── Express Mail — same auth, JSON API ───
function requireMailLogin(req, res, next) {
  if (req.session && req.session.userId) return next();
  return res.redirect("/dashboard.html");
}

app.get("/api/mail/inbox", requireMailLogin, (req, res) => {
  try {
    const mailDb = new Database(XYTROMAILING_DB, { readonly: true });
    const email = req.session.email || "";
    const rows = mailDb.prepare("SELECT id, sender_email, subject, is_read, received_at FROM mailbox_messages WHERE recipient_email = ? ORDER BY received_at DESC LIMIT 50").all(email);
    mailDb.close();
    res.json(rows);
  } catch(e) { res.status(500).json({ error: e.message }); }
});

app.get("/api/mail/message/:id", requireMailLogin, (req, res) => {
  try {
    const mailDb = new Database(XYTROMAILING_DB, { readonly: true });
    const row = mailDb.prepare("SELECT * FROM mailbox_messages WHERE id = ?").get(req.params.id);
    mailDb.close();
    if (!row) return res.status(404).json({ error: "Not found" });
    res.json(row);
  } catch(e) { res.status(500).json({ error: e.message }); }
});
