name = "my-app-hcl"
version = "invalid-version-format"
port = 80  # Port too low

database {
  host = "hcl-db.example.com"
  # Missing required port field
  name = "hcl_database"
} 