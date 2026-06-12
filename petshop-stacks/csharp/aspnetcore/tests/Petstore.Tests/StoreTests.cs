using System.Collections.Generic;
using Xunit;
using Petstore.Models;
using Petstore.Repositories;

namespace Petstore.Tests
{
    public class StoreTests
    {
        private static InMemoryPetstoreRepository CreateRepo() => new();

        private static Order SampleOrder(long id = 0) => new Order
        {
            Id       = id,
            PetId    = 1,
            Quantity = 2,
            Status   = Order.StatusEnum.PlacedEnum,
            Complete = false
        };

        [Fact]
        public void PlaceOrder_AssignsId_WhenIdIsZero()
        {
            var repo = CreateRepo();
            var result = repo.PlaceOrder(SampleOrder(0));
            Assert.True(result.Id > 0);
        }

        [Fact]
        public void PlaceOrder_PreservesProvidedId()
        {
            var repo = CreateRepo();
            var result = repo.PlaceOrder(SampleOrder(7));
            Assert.Equal(7, result.Id);
        }

        [Fact]
        public void GetOrderById_ReturnsPlacedOrder()
        {
            var repo = CreateRepo();
            repo.PlaceOrder(SampleOrder(1));
            var fetched = repo.GetOrderById(1);
            Assert.NotNull(fetched);
            Assert.Equal(Order.StatusEnum.PlacedEnum, fetched.Status);
        }

        [Fact]
        public void GetOrderById_ReturnsNull_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.Null(repo.GetOrderById(999));
        }

        [Fact]
        public void DeleteOrder_RemovesOrder()
        {
            var repo = CreateRepo();
            repo.PlaceOrder(SampleOrder(5));
            Assert.True(repo.DeleteOrder(5));
            Assert.Null(repo.GetOrderById(5));
        }

        [Fact]
        public void DeleteOrder_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.False(repo.DeleteOrder(999));
        }

        [Fact]
        public void GetInventory_ReflectsCurrentPetStatuses()
        {
            var repo = CreateRepo();
            repo.AddPet(new Pet { Id = 1, Name = "A", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.AvailableEnum });
            repo.AddPet(new Pet { Id = 2, Name = "B", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.AvailableEnum });
            repo.AddPet(new Pet { Id = 3, Name = "C", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.SoldEnum });

            var inventory = repo.GetInventory();
            Assert.Equal(2, inventory["available"]);
            Assert.Equal(1, inventory["sold"]);
        }

        [Fact]
        public void GetInventory_ReturnsEmpty_WhenNoPets()
        {
            var repo = CreateRepo();
            var inventory = repo.GetInventory();
            Assert.NotNull(inventory);
            Assert.Empty(inventory);
        }

        [Fact]
        public void PlaceOrder_UpdatesExistingOrder_OnConflict()
        {
            var repo = CreateRepo();
            repo.PlaceOrder(SampleOrder(10));
            var updated = SampleOrder(10);
            updated.Quantity = 99;
            repo.PlaceOrder(updated);
            Assert.Equal(99, repo.GetOrderById(10).Quantity);
        }
    }
}
