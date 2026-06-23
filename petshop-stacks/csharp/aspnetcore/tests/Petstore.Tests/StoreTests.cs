using System.Collections.Generic;
using System.Threading.Tasks;
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
        public async Task PlaceOrder_AssignsId_WhenIdIsZero()
        {
            var repo = CreateRepo();
            var result = await repo.PlaceOrder(SampleOrder(0));
            Assert.True(result.Id > 0);
        }

        [Fact]
        public async Task PlaceOrder_PreservesProvidedId()
        {
            var repo = CreateRepo();
            var result = await repo.PlaceOrder(SampleOrder(7));
            Assert.Equal(7, result.Id);
        }

        [Fact]
        public async Task GetOrderById_ReturnsPlacedOrder()
        {
            var repo = CreateRepo();
            await repo.PlaceOrder(SampleOrder(1));
            var fetched = await repo.GetOrderById(1);
            Assert.NotNull(fetched);
            Assert.Equal(Order.StatusEnum.PlacedEnum, fetched.Status);
        }

        [Fact]
        public async Task GetOrderById_ReturnsNull_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.Null(await repo.GetOrderById(999));
        }

        [Fact]
        public async Task DeleteOrder_RemovesOrder()
        {
            var repo = CreateRepo();
            await repo.PlaceOrder(SampleOrder(5));
            Assert.True(await repo.DeleteOrder(5));
            Assert.Null(await repo.GetOrderById(5));
        }

        [Fact]
        public async Task DeleteOrder_ReturnsFalse_WhenNotFound()
        {
            var repo = CreateRepo();
            Assert.False(await repo.DeleteOrder(999));
        }

        [Fact]
        public async Task GetInventory_ReflectsCurrentPetStatuses()
        {
            var repo = CreateRepo();
            await repo.AddPet(new Pet { Id = 1, Name = "A", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.AvailableEnum });
            await repo.AddPet(new Pet { Id = 2, Name = "B", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.AvailableEnum });
            await repo.AddPet(new Pet { Id = 3, Name = "C", PhotoUrls = new List<string>(), Status = Pet.StatusEnum.SoldEnum });

            var inventory = await repo.GetInventory();
            Assert.Equal(2, inventory["available"]);
            Assert.Equal(1, inventory["sold"]);
        }

        [Fact]
        public async Task GetInventory_ReturnsEmpty_WhenNoPets()
        {
            var repo = CreateRepo();
            var inventory = await repo.GetInventory();
            Assert.NotNull(inventory);
            Assert.Empty(inventory);
        }

        [Fact]
        public async Task PlaceOrder_UpdatesExistingOrder_OnConflict()
        {
            var repo = CreateRepo();
            await repo.PlaceOrder(SampleOrder(10));
            var updated = SampleOrder(10);
            updated.Quantity = 99;
            await repo.PlaceOrder(updated);
            Assert.Equal(99, (await repo.GetOrderById(10)).Quantity);
        }

        // ── ID sequencing: kills lines 120, 121, 122 mutations ───────────────────

        [Fact]
        public async Task PlaceOrder_AutoAssignsSequentialIds()
        {
            var repo = CreateRepo();
            var o1 = await repo.PlaceOrder(SampleOrder(0));
            var o2 = await repo.PlaceOrder(SampleOrder(0));
            Assert.Equal(1, o1.Id);
            Assert.Equal(2, o2.Id);
        }

        [Fact]
        public async Task PlaceOrder_SetsSequenceBeyondExplicitId()
        {
            // Kills: line 121 (>= vs > and negation), line 122 (+1 vs -1)
            var repo = CreateRepo();
            await repo.PlaceOrder(SampleOrder(5));
            var auto = await repo.PlaceOrder(SampleOrder(0));
            Assert.Equal(6, auto.Id);
        }

        [Fact]
        public async Task PlaceOrder_SetsSequenceWhenExplicitIdEqualsNextId()
        {
            // Kills: line 121 (>= vs >) — only differs when Id == _nextOrderId (both start at 1)
            var repo = CreateRepo();
            await repo.PlaceOrder(SampleOrder(1));
            var auto = await repo.PlaceOrder(SampleOrder(0));
            Assert.Equal(2, auto.Id);
        }
    }
}
