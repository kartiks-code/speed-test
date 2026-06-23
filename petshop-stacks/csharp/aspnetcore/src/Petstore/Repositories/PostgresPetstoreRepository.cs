using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Dapper;
using Newtonsoft.Json;
using Npgsql;
using Petstore.Models;

namespace Petstore.Repositories
{
    public class PostgresPetstoreRepository : IPetstoreRepository
    {
        private readonly NpgsqlDataSource _dataSource;

        public PostgresPetstoreRepository(NpgsqlDataSource dataSource)
        {
            _dataSource = dataSource;
        }

        public static string BuildConnectionString()
        {
            var databaseUrl = Environment.GetEnvironmentVariable("DATABASE_URL");
            if (!string.IsNullOrEmpty(databaseUrl))
                return ApplyPoolSize(databaseUrl);

            var host = Environment.GetEnvironmentVariable("POSTGRES_HOST") ?? "localhost";
            var port = Environment.GetEnvironmentVariable("POSTGRES_PORT") ?? "5434";
            var user = Environment.GetEnvironmentVariable("POSTGRES_USER") ?? "myuser";
            var password = Environment.GetEnvironmentVariable("POSTGRES_PASSWORD") ?? "mypassword";
            var db = Environment.GetEnvironmentVariable("POSTGRES_DB") ?? "csharp-aspnetcore";
            return ApplyPoolSize($"Host={host};Port={port};Username={user};Password={password};Database={db}");
        }

        // Appends "Maximum Pool Size" from PG_MAX_POOL_SIZE when set; unset keeps
        // the Npgsql default (100). Skips appending if the connection string
        // already specifies a pool size, so explicit DATABASE_URL settings win.
        public static string ApplyPoolSize(string connectionString)
        {
            var poolSize = Environment.GetEnvironmentVariable("PG_MAX_POOL_SIZE");
            if (string.IsNullOrEmpty(poolSize))
                return connectionString;

            var lower = connectionString.ToLowerInvariant();
            if (lower.Contains("maximum pool size") || lower.Contains("max pool size") || lower.Contains("maxpoolsize"))
                return connectionString;

            return $"{connectionString};Maximum Pool Size={poolSize}";
        }

        private async Task<NpgsqlConnection> OpenConnectionAsync() =>
            await _dataSource.OpenConnectionAsync();

        // ── Status helpers ─────────────────────────────────────────────

        private static string PetStatusToString(Pet.StatusEnum s) => s switch
        {
            Pet.StatusEnum.AvailableEnum => "available",
            Pet.StatusEnum.PendingEnum   => "pending",
            Pet.StatusEnum.SoldEnum      => "sold",
            _                            => null
        };

        private static Pet.StatusEnum StringToPetStatus(string s) => s switch
        {
            "pending" => Pet.StatusEnum.PendingEnum,
            "sold"    => Pet.StatusEnum.SoldEnum,
            _         => Pet.StatusEnum.AvailableEnum
        };

        private static string OrderStatusToString(Order.StatusEnum s) => s switch
        {
            Order.StatusEnum.ApprovedEnum  => "approved",
            Order.StatusEnum.DeliveredEnum => "delivered",
            Order.StatusEnum.PlacedEnum    => "placed",
            _                              => null
        };

        private static Order.StatusEnum StringToOrderStatus(string s) => s switch
        {
            "approved"  => Order.StatusEnum.ApprovedEnum,
            "delivered" => Order.StatusEnum.DeliveredEnum,
            _           => Order.StatusEnum.PlacedEnum
        };

        // ── Pet row → model ────────────────────────────────────────────

        private static Pet MapPetRow(dynamic row)
        {
            return new Pet
            {
                Id        = row.id,
                Name      = row.name,
                PhotoUrls = row.photo_urls != null
                    ? JsonConvert.DeserializeObject<List<string>>((string)row.photo_urls)
                    : new List<string>(),
                Tags = row.tags != null
                    ? JsonConvert.DeserializeObject<List<Tag>>((string)row.tags)
                    : null,
                Category = row.category != null
                    ? JsonConvert.DeserializeObject<Category>((string)row.category)
                    : null,
                Status = row.status != null
                    ? StringToPetStatus((string)row.status)
                    : default
            };
        }

        // ── Order row → model ──────────────────────────────────────────

        private static Order MapOrderRow(dynamic row)
        {
            return new Order
            {
                Id       = row.id,
                PetId    = row.pet_id,
                Quantity = row.quantity,
                ShipDate = row.ship_date != null ? (DateTime)row.ship_date : default,
                Status   = row.status != null ? StringToOrderStatus((string)row.status) : default,
                Complete = row.complete
            };
        }

        // ── User row → model ───────────────────────────────────────────

        private static User MapUserRow(dynamic row)
        {
            return new User
            {
                Id         = row.id ?? 0L,
                Username   = row.username,
                FirstName  = row.first_name,
                LastName   = row.last_name,
                Email      = row.email,
                Password   = row.password,
                Phone      = row.phone,
                UserStatus = row.user_status ?? 0
            };
        }

        // ══════════════════════ PET OPERATIONS ════════════════════════

        public async Task<Pet> AddPet(Pet pet)
        {
            await using var conn = await OpenConnectionAsync();
            var categoryJson  = pet.Category  != null ? JsonConvert.SerializeObject(pet.Category)  : null;
            var photoUrlsJson = JsonConvert.SerializeObject(pet.PhotoUrls ?? new List<string>());
            var tagsJson      = pet.Tags != null ? JsonConvert.SerializeObject(pet.Tags) : null;
            var statusStr     = PetStatusToString(pet.Status);

            // Single round-trip: COALESCE handles Id=0 (auto-assign via sequence) vs Id>0
            pet.Id = await conn.QuerySingleAsync<long>(
                @"INSERT INTO pet (id, name, category, photo_urls, tags, status)
                  VALUES (COALESCE(NULLIF(@Id, 0), nextval('pet_id_seq')), @Name, @Category::json, @PhotoUrls::json, @Tags::json, @Status::pet_status)
                  ON CONFLICT (id) DO UPDATE SET
                      name       = EXCLUDED.name,
                      category   = EXCLUDED.category,
                      photo_urls = EXCLUDED.photo_urls,
                      tags       = EXCLUDED.tags,
                      status     = EXCLUDED.status
                  RETURNING id",
                new { Id = pet.Id, Name = pet.Name, Category = categoryJson,
                      PhotoUrls = photoUrlsJson, Tags = tagsJson, Status = statusStr });

            return pet;
        }

        public async Task<bool> DeletePet(long petId)
        {
            await using var conn = await OpenConnectionAsync();
            var rows = await conn.ExecuteAsync("DELETE FROM pet WHERE id = @Id", new { Id = petId });
            return rows > 0;
        }

        public async Task<List<Pet>> FindPetsByStatus(string status)
        {
            await using var conn = await OpenConnectionAsync();
            IEnumerable<dynamic> rows;
            if (string.IsNullOrEmpty(status))
            {
                rows = await conn.QueryAsync<dynamic>(
                    "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet");
            }
            else
            {
                rows = await conn.QueryAsync<dynamic>(
                    "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet WHERE status::text = @Status",
                    new { Status = status });
            }
            return rows.Select(MapPetRow).ToList();
        }

        public async Task<List<Pet>> FindPetsByTags(List<string> tags)
        {
            await using var conn = await OpenConnectionAsync();

            if (tags == null || tags.Count == 0)
            {
                var allRows = await conn.QueryAsync<dynamic>(
                    "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet");
                return allRows.Select(MapPetRow).ToList();
            }

            // Push tag filtering to the database using json_array_elements — avoids full table scan in C#
            var rows = await conn.QueryAsync<dynamic>(
                @"SELECT id, name, category::text, photo_urls::text, tags::text, status::text
                  FROM pet
                  WHERE tags IS NOT NULL
                    AND EXISTS (
                        SELECT 1 FROM json_array_elements(tags) elem
                        WHERE elem->>'name' = ANY(@Tags)
                    )",
                new { Tags = tags.ToArray() });

            return rows.Select(MapPetRow).ToList();
        }

        public async Task<Pet> GetPetById(long petId)
        {
            await using var conn = await OpenConnectionAsync();
            var row = await conn.QueryFirstOrDefaultAsync<dynamic>(
                "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet WHERE id = @Id",
                new { Id = petId });
            return row == null ? null : MapPetRow(row);
        }

        public async Task<Pet> UpdatePet(Pet pet)
        {
            await using var conn = await OpenConnectionAsync();
            var categoryJson  = pet.Category  != null ? JsonConvert.SerializeObject(pet.Category)  : null;
            var photoUrlsJson = JsonConvert.SerializeObject(pet.PhotoUrls ?? new List<string>());
            var tagsJson      = pet.Tags != null ? JsonConvert.SerializeObject(pet.Tags) : null;
            var statusStr     = PetStatusToString(pet.Status);

            // Single round-trip: check affected rows instead of a prior SELECT
            var rows = await conn.ExecuteAsync(
                @"UPDATE pet SET name = @Name, category = @Category::json,
                      photo_urls = @PhotoUrls::json, tags = @Tags::json, status = @Status::pet_status
                  WHERE id = @Id",
                new { Id = pet.Id, Name = pet.Name, Category = categoryJson,
                      PhotoUrls = photoUrlsJson, Tags = tagsJson, Status = statusStr });

            return rows > 0 ? pet : null;
        }

        public async Task<bool> UpdatePetWithForm(long petId, string name, string status)
        {
            await using var conn = await OpenConnectionAsync();

            // Single round-trip: COALESCE keeps existing value when parameter is null
            var rows = await conn.ExecuteAsync(
                @"UPDATE pet SET
                      name   = COALESCE(@Name, name),
                      status = CASE WHEN @Status IS NULL THEN status ELSE @Status::pet_status END
                  WHERE id = @Id",
                new { Id = petId, Name = name, Status = status });

            return rows > 0;
        }

        public async Task<ApiResponse> UploadFile(long petId, string additionalMetadata, Stream fileData)
        {
            byte[] bytes;
            using (var ms = new MemoryStream())
            {
                await fileData.CopyToAsync(ms);
                bytes = ms.ToArray();
            }

            await using var conn = await OpenConnectionAsync();

            // Single round-trip CTE: verifies pet exists, gets next sequence id, and inserts —
            // returns null rows (→ null) when the pet does not exist.
            var photoId = await conn.QuerySingleOrDefaultAsync<long?>(
                @"WITH check_pet AS (
                      SELECT id FROM pet WHERE id = @PetId
                  ),
                  new_id AS (
                      SELECT nextval('pet_photo_id_seq') AS id FROM check_pet
                  )
                  INSERT INTO pet_photo (id, pet_id, metadata, content)
                  SELECT id, @PetId, @Metadata, @Content FROM new_id
                  RETURNING id",
                new { PetId = petId, Metadata = additionalMetadata, Content = bytes });

            if (photoId == null) return null;

            return new ApiResponse
            {
                Code    = 200,
                Message = $"File uploaded, {bytes.Length} bytes stored."
            };
        }

        // ══════════════════════ STORE OPERATIONS ══════════════════════

        public async Task<bool> DeleteOrder(long orderId)
        {
            await using var conn = await OpenConnectionAsync();
            var rows = await conn.ExecuteAsync("DELETE FROM \"order\" WHERE id = @Id", new { Id = orderId });
            return rows > 0;
        }

        public async Task<Dictionary<string, int>> GetInventory()
        {
            await using var conn = await OpenConnectionAsync();
            var rows = await conn.QueryAsync<dynamic>(
                "SELECT status::text AS status, COUNT(*)::int AS cnt FROM pet GROUP BY status");
            var result = new Dictionary<string, int>();
            foreach (var r in rows)
            {
                var key = (string)r.status ?? "unknown";
                result[key] = (int)r.cnt;
            }
            return result;
        }

        public async Task<Order> GetOrderById(long orderId)
        {
            await using var conn = await OpenConnectionAsync();
            var row = await conn.QueryFirstOrDefaultAsync<dynamic>(
                "SELECT id, pet_id, quantity, ship_date, status::text, complete FROM \"order\" WHERE id = @Id",
                new { Id = orderId });
            return row == null ? null : MapOrderRow(row);
        }

        public async Task<Order> PlaceOrder(Order order)
        {
            await using var conn = await OpenConnectionAsync();
            var statusStr = OrderStatusToString(order.Status);

            // Single round-trip: COALESCE handles Id=0 (auto-assign via sequence) vs Id>0
            order.Id = await conn.QuerySingleAsync<long>(
                @"INSERT INTO ""order"" (id, pet_id, quantity, ship_date, status, complete)
                  VALUES (COALESCE(NULLIF(@Id, 0), nextval('order_id_seq')), @PetId, @Quantity, @ShipDate, @Status::order_status, @Complete)
                  ON CONFLICT (id) DO UPDATE SET
                      pet_id    = EXCLUDED.pet_id,
                      quantity  = EXCLUDED.quantity,
                      ship_date = EXCLUDED.ship_date,
                      status    = EXCLUDED.status,
                      complete  = EXCLUDED.complete
                  RETURNING id",
                new { Id = order.Id, PetId = order.PetId, order.Quantity,
                      ShipDate = order.ShipDate == default ? (DateTime?)null : order.ShipDate,
                      Status = statusStr, order.Complete });

            return order;
        }

        // ══════════════════════ USER OPERATIONS ═══════════════════════

        public async Task<User> CreateUser(User user)
        {
            await using var conn = await OpenConnectionAsync();
            return await InsertUserAsync(conn, user);
        }

        public async Task<User> CreateUsersWithListInput(List<User> users)
        {
            if (users == null || users.Count == 0)
                return null;

            // Single connection shared across all inserts — avoids N connection acquisitions
            await using var conn = await OpenConnectionAsync();
            User last = null;
            foreach (var u in users)
                last = await InsertUserAsync(conn, u);
            return last;
        }

        private static async Task<User> InsertUserAsync(NpgsqlConnection conn, User user)
        {
            // Single round-trip: COALESCE handles Id=0 (auto-assign via sequence) vs Id>0
            user.Id = await conn.QuerySingleAsync<long>(
                @"INSERT INTO ""user"" (id, username, first_name, last_name, email, password, phone, user_status)
                  VALUES (COALESCE(NULLIF(@Id, 0), nextval('user_id_seq')), @Username, @FirstName, @LastName, @Email, @Password, @Phone, @UserStatus)
                  ON CONFLICT (username) DO UPDATE SET
                      id          = EXCLUDED.id,
                      first_name  = EXCLUDED.first_name,
                      last_name   = EXCLUDED.last_name,
                      email       = EXCLUDED.email,
                      password    = EXCLUDED.password,
                      phone       = EXCLUDED.phone,
                      user_status = EXCLUDED.user_status
                  RETURNING id",
                new { user.Id, user.Username, FirstName = user.FirstName,
                      LastName = user.LastName, user.Email, user.Password,
                      user.Phone, UserStatus = user.UserStatus });
            return user;
        }

        public async Task<bool> DeleteUser(string username)
        {
            await using var conn = await OpenConnectionAsync();
            var rows = await conn.ExecuteAsync(
                "DELETE FROM \"user\" WHERE username = @Username", new { Username = username });
            return rows > 0;
        }

        public async Task<User> GetUserByName(string username)
        {
            await using var conn = await OpenConnectionAsync();
            var row = await conn.QueryFirstOrDefaultAsync<dynamic>(
                "SELECT id, username, first_name, last_name, email, password, phone, user_status FROM \"user\" WHERE username = @Username",
                new { Username = username });
            return row == null ? null : MapUserRow(row);
        }

        public Task<string> LoginUser(string username, string password)
        {
            return Task.FromResult("logged-in");
        }

        public Task LogoutUser()
        {
            return Task.CompletedTask;
        }

        public async Task<bool> UpdateUser(string username, User user)
        {
            await using var conn = await OpenConnectionAsync();
            var rows = await conn.ExecuteAsync(
                @"UPDATE ""user"" SET first_name = @FirstName, last_name = @LastName,
                      email = @Email, password = @Password, phone = @Phone, user_status = @UserStatus
                  WHERE username = @Username",
                new { Username = username, FirstName = user.FirstName,
                      LastName = user.LastName, user.Email, user.Password,
                      user.Phone, UserStatus = user.UserStatus });
            return rows > 0;
        }
    }
}
