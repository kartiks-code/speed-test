using System.Collections.Generic;
using System.IO;
using System.Text;
using Xunit;
using Petstore.Models;
using Petstore.Repositories;

namespace Petstore.Tests
{
    public class PetTests
    {
        private static InMemoryPetstoreRepository CreateRepo() => new();

        private static Pet SamplePet(long id = 0) => new Pet
        {
            Id        = id,
            Name      = "Buddy",
            PhotoUrls = new List<string> { "http://example.com/photo.jpg" },
            Status    = Pet.StatusEnum.AvailableEnum,
            Category  = new Category { Id = 1, Name = "Dogs" },
            Tags      = new List<Tag> { new Tag { Id = 1, Name = "friendly" } }
        };

        [Fact]
        public void AddPet_AssignsId_WhenIdIsZero()
        {
            var repo = CreateRepo();
            var pet = SamplePet(0);
            var result = repo.AddPet(pet);
            Assert.True(result.Id > 0);
        }

        [Fact]
        public void AddPet_PreservesProvidedId()
        {
            var repo = CreateRepo();
            var pet = SamplePet(42);
            var result = repo.AddPet(pet);
            Assert.Equal(42, result.Id);
        }

        [Fact]
        public void GetPetById_ReturnsCreatedPet()
        {
            var repo = CreateRepo();
            var added = repo.AddPet(SamplePet(1));
            var fetched = repo.GetPetById(1);
            Assert.NotNull(fetched);
            Assert.Equal("Buddy", fetched.Name);
        }

        [Fact]
        public void GetPetById_ReturnsNull_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.Null(repo.GetPetById(999));
        }

        [Fact]
        public void UpdatePet_UpdatesExistingPet()
        {
            var repo = CreateRepo();
            var added = repo.AddPet(SamplePet(5));
            added.Name = "Rex";
            var updated = repo.UpdatePet(added);
            Assert.Equal("Rex", updated.Name);
            Assert.Equal("Rex", repo.GetPetById(5).Name);
        }

        [Fact]
        public void UpdatePet_ReturnsNull_WhenNotFound()
        {
            var repo = CreateRepo();
            var result = repo.UpdatePet(SamplePet(999));
            Assert.Null(result);
        }

        [Fact]
        public void DeletePet_RemovesPet()
        {
            var repo = CreateRepo();
            repo.AddPet(SamplePet(10));
            Assert.True(repo.DeletePet(10));
            Assert.Null(repo.GetPetById(10));
        }

        [Fact]
        public void DeletePet_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.False(repo.DeletePet(999));
        }

        [Fact]
        public void FindPetsByStatus_ReturnsMatchingPets()
        {
            var repo = CreateRepo();
            repo.AddPet(new Pet { Id = 1, Name = "A", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.AvailableEnum });
            repo.AddPet(new Pet { Id = 2, Name = "B", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.SoldEnum });
            var available = repo.FindPetsByStatus("available");
            Assert.Single(available);
            Assert.Equal("A", available[0].Name);
        }

        [Fact]
        public void FindPetsByStatus_ReturnsAll_WhenStatusEmpty()
        {
            var repo = CreateRepo();
            repo.AddPet(new Pet { Id = 1, Name = "A", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.AvailableEnum });
            repo.AddPet(new Pet { Id = 2, Name = "B", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.SoldEnum });
            var all = repo.FindPetsByStatus("");
            Assert.Equal(2, all.Count);
        }

        [Fact]
        public void FindPetsByTags_ReturnsMatchingPets()
        {
            var repo = CreateRepo();
            repo.AddPet(new Pet
            {
                Id = 1, Name = "A", PhotoUrls = new List<string>(),
                Tags = new List<Tag> { new Tag { Id = 1, Name = "cute" } }
            });
            repo.AddPet(new Pet { Id = 2, Name = "B", PhotoUrls = new List<string>(), Tags = null });

            var result = repo.FindPetsByTags(new List<string> { "cute" });
            Assert.Single(result);
        }

        [Fact]
        public void UpdatePetWithForm_UpdatesFields()
        {
            var repo = CreateRepo();
            repo.AddPet(SamplePet(20));
            var ok = repo.UpdatePetWithForm(20, "NewName", "sold");
            Assert.True(ok);
            var pet = repo.GetPetById(20);
            Assert.Equal("NewName", pet.Name);
            Assert.Equal(Pet.StatusEnum.SoldEnum, pet.Status);
        }

        [Fact]
        public void UpdatePetWithForm_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.False(repo.UpdatePetWithForm(999, "name", "available"));
        }

        [Fact]
        public void UploadFile_ReturnsApiResponse_WhenPetExists()
        {
            var repo = CreateRepo();
            repo.AddPet(SamplePet(30));
            var bytes = Encoding.UTF8.GetBytes("fake-image-data");
            var stream = new MemoryStream(bytes);
            var response = repo.UploadFile(30, "meta", stream);
            Assert.NotNull(response);
            Assert.Contains("bytes", response.Message);
        }

        [Fact]
        public void UploadFile_ReturnsNull_WhenPetNotFound()
        {
            var repo = CreateRepo();
            var response = repo.UploadFile(999, null, new MemoryStream());
            Assert.Null(response);
        }
    }
}
