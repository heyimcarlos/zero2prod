# Newsletter API

## Notes

- Before migrating the database, the app needs to have `trusted sources` disabled.
- After deployment to the digital ocean's app platform, migrations have to be applied
manually with the following command:

```bash
DATABASE_URL=<database_url> sqlx migrate run
```
