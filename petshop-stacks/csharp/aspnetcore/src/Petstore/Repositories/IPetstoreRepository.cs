using System.Collections.Generic;
using System.IO;
using Petstore.Models;

namespace Petstore.Repositories
{
    public interface IPetstoreRepository
    {
        // Pet
        Pet AddPet(Pet pet);
        bool DeletePet(long petId);
        List<Pet> FindPetsByStatus(string status);
        List<Pet> FindPetsByTags(List<string> tags);
        Pet GetPetById(long petId);
        Pet UpdatePet(Pet pet);
        bool UpdatePetWithForm(long petId, string name, string status);
        ApiResponse UploadFile(long petId, string additionalMetadata, Stream fileData);

        // Store
        bool DeleteOrder(long orderId);
        Dictionary<string, int> GetInventory();
        Order GetOrderById(long orderId);
        Order PlaceOrder(Order order);

        // User
        User CreateUser(User user);
        User CreateUsersWithListInput(List<User> users);
        bool DeleteUser(string username);
        User GetUserByName(string username);
        string LoginUser(string username, string password);
        void LogoutUser();
        bool UpdateUser(string username, User user);
    }
}
