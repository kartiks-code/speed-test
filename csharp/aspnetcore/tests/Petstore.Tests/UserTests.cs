using System.Collections.Generic;
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
        public void CreateUser_AssignsId_WhenIdIsZero()
        {
            var repo = CreateRepo();
            var result = repo.CreateUser(SampleUser());
            Assert.True(result.Id > 0);
        }

        [Fact]
        public void GetUserByName_ReturnsCreatedUser()
        {
            var repo = CreateRepo();
            repo.CreateUser(SampleUser("alice"));
            var fetched = repo.GetUserByName("alice");
            Assert.NotNull(fetched);
            Assert.Equal("John", fetched.FirstName);
        }

        [Fact]
        public void GetUserByName_ReturnsNull_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.Null(repo.GetUserByName("nobody"));
        }

        [Fact]
        public void UpdateUser_UpdatesFields()
        {
            var repo = CreateRepo();
            repo.CreateUser(SampleUser("bob"));
            var updated = SampleUser("bob");
            updated.FirstName = "Bobby";
            updated.Email = "bobby@example.com";
            repo.UpdateUser("bob", updated);
            var fetched = repo.GetUserByName("bob");
            Assert.Equal("Bobby", fetched.FirstName);
            Assert.Equal("bobby@example.com", fetched.Email);
        }

        [Fact]
        public void UpdateUser_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            var ok = repo.UpdateUser("ghost", SampleUser("ghost"));
            Assert.False(ok);
        }

        [Fact]
        public void DeleteUser_RemovesUser()
        {
            var repo = CreateRepo();
            repo.CreateUser(SampleUser("carol"));
            Assert.True(repo.DeleteUser("carol"));
            Assert.Null(repo.GetUserByName("carol"));
        }

        [Fact]
        public void DeleteUser_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.False(repo.DeleteUser("nobody"));
        }

        [Fact]
        public void CreateUsersWithListInput_CreatesAll_ReturnsLast()
        {
            var repo = CreateRepo();
            var users = new List<User>
            {
                SampleUser("u1"),
                SampleUser("u2"),
                SampleUser("u3")
            };
            var last = repo.CreateUsersWithListInput(users);
            Assert.Equal("u3", last.Username);
            Assert.NotNull(repo.GetUserByName("u1"));
            Assert.NotNull(repo.GetUserByName("u2"));
        }

        [Fact]
        public void LoginUser_ReturnsToken()
        {
            var repo = CreateRepo();
            var token = repo.LoginUser("johndoe", "secret");
            Assert.NotNull(token);
            Assert.NotEmpty(token);
        }

        [Fact]
        public void LogoutUser_IsNoOp()
        {
            var repo = CreateRepo();
            repo.LogoutUser();
        }

        [Fact]
        public void CreateUser_Upserts_WhenUsernameExists()
        {
            var repo = CreateRepo();
            var u1 = SampleUser("dave");
            repo.CreateUser(u1);
            var u2 = SampleUser("dave");
            u2.FirstName = "David";
            repo.CreateUser(u2);
            Assert.Equal("David", repo.GetUserByName("dave").FirstName);
        }
    }
}
