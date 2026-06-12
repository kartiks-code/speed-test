using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Dapper;
using Newtonsoft.Json;
using Npgsql;
using Petstore.Models;

namespace Petstore.Repositories
{
    public class PostgresPetstoreRepository : IPetstoreRepository
    {
        private readonly string _connectionString;

        public PostgresPetstoreRepository()
        {
            _connectionString = BuildConnectionString();
        }

        private static string BuildConnectionString()
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
        private static string ApplyPoolSize(string connectionString)
        {
            var poolSize = Environment.GetEnvironmentVariable("PG_MAX_POOL_SIZE");
            if (string.IsNullOrEmpty(poolSize))
                return connectionString;

            var lower = connectionString.ToLowerInvariant();
            if (lower.Contains("maximum pool size") || lower.Contains("max pool size") || lower.Contains("maxpoolsize"))
                return connectionString;

            return $"{connectionString};Maximum Pool Size={poolSize}";
        }

        private NpgsqlConnection OpenConnection() =>
            new NpgsqlConnection(_connectionString);

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
            var pet = new Pet
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
            return pet;
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

        public Pet AddPet(Pet pet)
        {
            using var conn = OpenConnection();
            if (pet.Id == 0)
                pet.Id = conn.QuerySingle<long>("SELECT nextval('pet_id_seq')");

            var categoryJson  = pet.Category  != null ? JsonConvert.SerializeObject(pet.Category)  : null;
            var photoUrlsJson = JsonConvert.SerializeObject(pet.PhotoUrls ?? new List<string>());
            var tagsJson      = pet.Tags != null ? JsonConvert.SerializeObject(pet.Tags) : null;
            var statusStr     = PetStatusToString(pet.Status);

            conn.Execute(
                @"INSERT INTO pet (id, name, category, photo_urls, tags, status)
                  VALUES (@Id, @Name, @Category::json, @PhotoUrls::json, @Tags::json, @Status::pet_status)
                  ON CONFLICT (id) DO UPDATE SET
                      name       = EXCLUDED.name,
                      category   = EXCLUDED.category,
                      photo_urls = EXCLUDED.photo_urls,
                      tags       = EXCLUDED.tags,
                      status     = EXCLUDED.status",
                new { Id = pet.Id, Name = pet.Name, Category = categoryJson,
                      PhotoUrls = photoUrlsJson, Tags = tagsJson, Status = statusStr });

            return pet;
        }

        public bool DeletePet(long petId)
        {
            using var conn = OpenConnection();
            var rows = conn.Execute("DELETE FROM pet WHERE id = @Id", new { Id = petId });
            return rows > 0;
        }

        public List<Pet> FindPetsByStatus(string status)
        {
            using var conn = OpenConnection();
            IEnumerable<dynamic> rows;
            if (string.IsNullOrEmpty(status))
            {
                rows = conn.Query<dynamic>(
                    "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet");
            }
            else
            {
                rows = conn.Query<dynamic>(
                    "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet WHERE status::text = @Status",
                    new { Status = status });
            }
            return rows.Select(MapPetRow).ToList();
        }

        public List<Pet> FindPetsByTags(List<string> tags)
        {
            using var conn = OpenConnection();
            var allPets = conn.Query<dynamic>(
                "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet")
                .Select(MapPetRow)
                .ToList();

            if (tags == null || tags.Count == 0)
                return allPets;

            return allPets.Where(p =>
                p.Tags != null && p.Tags.Any(t => tags.Contains(t.Name))
            ).ToList();
        }

        public Pet GetPetById(long petId)
        {
            using var conn = OpenConnection();
            var row = conn.QueryFirstOrDefault<dynamic>(
                "SELECT id, name, category::text, photo_urls::text, tags::text, status::text FROM pet WHERE id = @Id",
                new { Id = petId });
            return row == null ? null : MapPetRow(row);
        }

        public Pet UpdatePet(Pet pet)
        {
            using var conn = OpenConnection();
            var existing = conn.QueryFirstOrDefault<dynamic>(
                "SELECT id FROM pet WHERE id = @Id", new { Id = pet.Id });
            if (existing == null) return null;

            var categoryJson  = pet.Category  != null ? JsonConvert.SerializeObject(pet.Category)  : null;
            var photoUrlsJson = JsonConvert.SerializeObject(pet.PhotoUrls ?? new List<string>());
            var tagsJson      = pet.Tags != null ? JsonConvert.SerializeObject(pet.Tags) : null;
            var statusStr     = PetStatusToString(pet.Status);

            conn.Execute(
                @"UPDATE pet SET name = @Name, category = @Category::json,
                      photo_urls = @PhotoUrls::json, tags = @Tags::json, status = @Status::pet_status
                  WHERE id = @Id",
                new { Id = pet.Id, Name = pet.Name, Category = categoryJson,
                      PhotoUrls = photoUrlsJson, Tags = tagsJson, Status = statusStr });

            return pet;
        }

        public bool UpdatePetWithForm(long petId, string name, string status)
        {
            using var conn = OpenConnection();
            var existing = conn.QueryFirstOrDefault<dynamic>(
                "SELECT id, name, status::text FROM pet WHERE id = @Id", new { Id = petId });
            if (existing == null) return false;

            var newName   = name   ?? (string)existing.name;
            var newStatus = status ?? (string)existing.status;

            conn.Execute(
                "UPDATE pet SET name = @Name, status = @Status::pet_status WHERE id = @Id",
                new { Id = petId, Name = newName, Status = newStatus });
            return true;
        }

        public ApiResponse UploadFile(long petId, string additionalMetadata, Stream fileData)
        {
            using var conn = OpenConnection();
            var petExists = conn.QueryFirstOrDefault<dynamic>(
                "SELECT id FROM pet WHERE id = @Id", new { Id = petId });
            if (petExists == null) return null;

            byte[] bytes;
            using (var ms = new MemoryStream())
            {
                fileData.CopyTo(ms);
                bytes = ms.ToArray();
            }

            var photoId = conn.QuerySingle<long>(
                "SELECT nextval('pet_photo_id_seq')");

            conn.Execute(
                @"INSERT INTO pet_photo (id, pet_id, metadata, content)
                  VALUES (@Id, @PetId, @Metadata, @Content)
                  ON CONFLICT (id) DO UPDATE SET
                      pet_id   = EXCLUDED.pet_id,
                      metadata = EXCLUDED.metadata,
                      content  = EXCLUDED.content",
                new { Id = photoId, PetId = petId, Metadata = additionalMetadata, Content = bytes });

            return new ApiResponse
            {
                Code    = 200,
                Message = $"File uploaded, {bytes.Length} bytes stored."
            };
        }

        // ══════════════════════ STORE OPERATIONS ══════════════════════

        public bool DeleteOrder(long orderId)
        {
            using var conn = OpenConnection();
            var rows = conn.Execute("DELETE FROM \"order\" WHERE id = @Id", new { Id = orderId });
            return rows > 0;
        }

        public Dictionary<string, int> GetInventory()
        {
            using var conn = OpenConnection();
            var rows = conn.Query<dynamic>(
                "SELECT status::text AS status, COUNT(*)::int AS cnt FROM pet GROUP BY status");
            var result = new Dictionary<string, int>();
            foreach (var r in rows)
            {
                var key = (string)r.status ?? "unknown";
                result[key] = (int)r.cnt;
            }
            return result;
        }

        public Order GetOrderById(long orderId)
        {
            using var conn = OpenConnection();
            var row = conn.QueryFirstOrDefault<dynamic>(
                "SELECT id, pet_id, quantity, ship_date, status::text, complete FROM \"order\" WHERE id = @Id",
                new { Id = orderId });
            return row == null ? null : MapOrderRow(row);
        }

        public Order PlaceOrder(Order order)
        {
            using var conn = OpenConnection();
            if (order.Id == 0)
                order.Id = conn.QuerySingle<long>(
                    "SELECT nextval('order_id_seq')");

            var statusStr = OrderStatusToString(order.Status);
            conn.Execute(
                @"INSERT INTO ""order"" (id, pet_id, quantity, ship_date, status, complete)
                  VALUES (@Id, @PetId, @Quantity, @ShipDate, @Status::order_status, @Complete)
                  ON CONFLICT (id) DO UPDATE SET
                      pet_id    = EXCLUDED.pet_id,
                      quantity  = EXCLUDED.quantity,
                      ship_date = EXCLUDED.ship_date,
                      status    = EXCLUDED.status,
                      complete  = EXCLUDED.complete",
                new { order.Id, PetId = order.PetId, order.Quantity,
                      ShipDate = order.ShipDate == default ? (DateTime?)null : order.ShipDate,
                      Status = statusStr, order.Complete });

            return order;
        }

        // ══════════════════════ USER OPERATIONS ═══════════════════════

        public User CreateUser(User user)
        {
            using var conn = OpenConnection();
            if (user.Id == 0)
                user.Id = conn.QuerySingle<long>(
                    "SELECT nextval('user_id_seq')");

            conn.Execute(
                @"INSERT INTO ""user"" (id, username, first_name, last_name, email, password, phone, user_status)
                  VALUES (@Id, @Username, @FirstName, @LastName, @Email, @Password, @Phone, @UserStatus)
                  ON CONFLICT (username) DO UPDATE SET
                      id          = EXCLUDED.id,
                      first_name  = EXCLUDED.first_name,
                      last_name   = EXCLUDED.last_name,
                      email       = EXCLUDED.email,
                      password    = EXCLUDED.password,
                      phone       = EXCLUDED.phone,
                      user_status = EXCLUDED.user_status",
                new { user.Id, user.Username, FirstName = user.FirstName,
                      LastName = user.LastName, user.Email, user.Password,
                      user.Phone, UserStatus = user.UserStatus });

            return user;
        }

        public User CreateUsersWithListInput(List<User> users)
        {
            if (users == null || users.Count == 0)
                return null;
            User last = null;
            foreach (var u in users)
                last = CreateUser(u);
            return last;
        }

        public bool DeleteUser(string username)
        {
            using var conn = OpenConnection();
            var rows = conn.Execute(
                "DELETE FROM \"user\" WHERE username = @Username", new { Username = username });
            return rows > 0;
        }

        public User GetUserByName(string username)
        {
            using var conn = OpenConnection();
            var row = conn.QueryFirstOrDefault<dynamic>(
                "SELECT id, username, first_name, last_name, email, password, phone, user_status FROM \"user\" WHERE username = @Username",
                new { Username = username });
            return row == null ? null : MapUserRow(row);
        }

        public string LoginUser(string username, string password)
        {
            return "logged-in";
        }

        public void LogoutUser()
        {
        }

        public bool UpdateUser(string username, User user)
        {
            using var conn = OpenConnection();
            var rows = conn.Execute(
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
