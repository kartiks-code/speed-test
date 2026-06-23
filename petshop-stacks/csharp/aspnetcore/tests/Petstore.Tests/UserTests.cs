using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;
using Petstore.Models;
using Petstore.Repositories;

namespace Petstore.Tests
{
    public class UserTests
    {
        private static InMemoryPetstoreRepository CreateRepo() => new();

        private static User SampleUser(string username = "johndoe") => new User
        {
            Id         = 0,
            Username   = username,
            FirstName  = "John",
            LastName   = "Doe",
            Email      = "john@example.com",
            Password   = "secret",
            Phone      = "555-1234",
            UserStatus = 1
        };

        [Fact]
        public async Task CreateUser_AssignsId_WhenIdIsZero()
        {
            var repo = CreateRepo();
            var result = await repo.CreateUser(SampleUser());
            Assert.True(result.Id > 0);
        }

        [Fact]
        public async Task GetUserByName_ReturnsCreatedUser()
        {
            var repo = CreateRepo();
            await repo.CreateUser(SampleUser("alice"));
            var fetched = await repo.GetUserByName("alice");
            Assert.NotNull(fetched);
            Assert.Equal("John", fetched.FirstName);
        }

        [Fact]
        public async Task GetUserByName_ReturnsNull_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.Null(await repo.GetUserByName("nobody"));
        }

        [Fact]
        public async Task UpdateUser_UpdatesFields()
        {
            var repo = CreateRepo();
            await repo.CreateUser(SampleUser("bob"));
            var updated = SampleUser("bob");
            updated.FirstName = "Bobby";
            updated.Email = "bobby@example.com";
            await repo.UpdateUser("bob", updated);
            var fetched = await repo.GetUserByName("bob");
            Assert.Equal("Bobby", fetched.FirstName);
            Assert.Equal("bobby@example.com", fetched.Email);
        }

        [Fact]
        public async Task UpdateUser_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            var ok = await repo.UpdateUser("ghost", SampleUser("ghost"));
            Assert.False(ok);
        }

        [Fact]
        public async Task DeleteUser_RemovesUser()
        {
            var repo = CreateRepo();
            await repo.CreateUser(SampleUser("carol"));
            Assert.True(await repo.DeleteUser("carol"));
            Assert.Null(await repo.GetUserByName("carol"));
        }

        [Fact]
        public async Task DeleteUser_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.False(await repo.DeleteUser("nobody"));
        }

        [Fact]
        public async Task CreateUsersWithListInput_CreatesAll_ReturnsLast()
        {
            var repo = CreateRepo();
            var users = new List<User>
            {
                SampleUser("u1"),
                SampleUser("u2"),
                SampleUser("u3")
            };
            var last = await repo.CreateUsersWithListInput(users);
            Assert.Equal("u3", last.Username);
            Assert.NotNull(await repo.GetUserByName("u1"));
            Assert.NotNull(await repo.GetUserByName("u2"));
        }

        [Fact]
        public async Task LoginUser_ReturnsToken()
        {
            var repo = CreateRepo();
            var token = await repo.LoginUser("johndoe", "secret");
            Assert.NotNull(token);
            Assert.NotEmpty(token);
        }

        [Fact]
        public async Task LogoutUser_IsNoOp()
        {
            var repo = CreateRepo();
            await repo.LogoutUser();
        }

        [Fact]
        public async Task CreateUser_Upserts_WhenUsernameExists()
        {
            var repo = CreateRepo();
            var u1 = SampleUser("dave");
            await repo.CreateUser(u1);
            var u2 = SampleUser("dave");
            u2.FirstName = "David";
            await repo.CreateUser(u2);
            Assert.Equal("David", (await repo.GetUserByName("dave")).FirstName);
        }

        // ── ID sequencing: kills line 133 mutation, covers lines 134–135 ─────────

        [Fact]
        public async Task CreateUser_AutoAssignsSequentialIds()
        {
            var repo = CreateRepo();
            var u1 = await repo.CreateUser(SampleUser("alpha"));
            var u2 = await repo.CreateUser(SampleUser("beta"));
            Assert.Equal(1, u1.Id);
            Assert.Equal(2, u2.Id);
        }

        [Fact]
        public async Task CreateUser_SetsSequenceBeyondExplicitId()
        {
            // Covers NoCoverage lines 134-135 (>= vs >, negate, +1 vs -1)
            var repo = CreateRepo();
            var explicit_ = SampleUser("withid");
            explicit_.Id = 100;
            await repo.CreateUser(explicit_);
            var auto = await repo.CreateUser(SampleUser("auto"));
            Assert.Equal(101, auto.Id);
        }

        [Fact]
        public async Task CreateUser_SetsSequenceWhenExplicitIdEqualsNextId()
        {
            // Kills: line 134 (>= vs >) — only differs when Id == _nextUserId (both start at 1)
            var repo = CreateRepo();
            var explicit_ = SampleUser("explicit");
            explicit_.Id = 1;
            await repo.CreateUser(explicit_);
            var auto = await repo.CreateUser(SampleUser("auto"));
            Assert.Equal(2, auto.Id);
        }

        // ── CreateUsersWithListInput empty list: kills line 143 (|| vs &&) ────────

        [Fact]
        public async Task CreateUsersWithListInput_ReturnsNull_ForEmptyList()
        {
            var repo = CreateRepo();
            var result = await repo.CreateUsersWithListInput(new List<User>());
            Assert.Null(result);
        }

        [Fact]
        public async Task CreateUsersWithListInput_ReturnsNull_ForNullList()
        {
            // Kills line 143 (|| vs &&): with &&, null input causes NullReferenceException on .Count
            var repo = CreateRepo();
            var result = await repo.CreateUsersWithListInput(null);
            Assert.Null(result);
        }

        // ── UpdateUser return value: kills line 181 (return true → false) ─────────

        [Fact]
        public async Task UpdateUser_ReturnsTrue_WhenUserExists()
        {
            var repo = CreateRepo();
            await repo.CreateUser(SampleUser("test"));
            var ok = await repo.UpdateUser("test", SampleUser("test"));
            Assert.True(ok);
        }
    }
}
