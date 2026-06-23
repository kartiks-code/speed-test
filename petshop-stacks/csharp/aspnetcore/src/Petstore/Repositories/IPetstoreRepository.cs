using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using Petstore.Models;

namespace Petstore.Repositories
{
    public interface IPetstoreRepository
    {
        // Pet
        Task<Pet> AddPet(Pet pet);
        Task<bool> DeletePet(long petId);
        Task<List<Pet>> FindPetsByStatus(string status);
        Task<List<Pet>> FindPetsByTags(List<string> tags);
        Task<Pet> GetPetById(long petId);
        Task<Pet> UpdatePet(Pet pet);
        Task<bool> UpdatePetWithForm(long petId, string name, string status);
        Task<ApiResponse> UploadFile(long petId, string additionalMetadata, Stream fileData);

        // Store
        Task<bool> DeleteOrder(long orderId);
        Task<Dictionary<string, int>> GetInventory();
        Task<Order> GetOrderById(long orderId);
        Task<Order> PlaceOrder(Order order);

        // User
        Task<User> CreateUser(User user);
        Task<User> CreateUsersWithListInput(List<User> users);
        Task<bool> DeleteUser(string username);
        Task<User> GetUserByName(string username);
        Task<string> LoginUser(string username, string password);
        Task LogoutUser();
        Task<bool> UpdateUser(string username, User user);
    }
}
