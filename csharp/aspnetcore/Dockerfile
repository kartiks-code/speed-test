FROM mcr.microsoft.com/dotnet/core/aspnet:8.0-buster-slim AS base
WORKDIR /app
EXPOSE 80
EXPOSE 443

FROM mcr.microsoft.com/dotnet/core/sdk:8.0-buster AS build
WORKDIR /src
COPY ["src/Petstore/Petstore.csproj", "Petstore/"]
RUN dotnet restore "Petstore/Petstore.csproj"
COPY src/Petstore/ Petstore/
WORKDIR "Petstore"
RUN dotnet build "Petstore.csproj" -c Release -o /app/build

FROM build AS publish
RUN dotnet publish "Petstore.csproj" -c Release -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENV ASPNETCORE_URLS=http://+:8080
EXPOSE 8080
ENTRYPOINT ["dotnet", "Petstore.dll"]
