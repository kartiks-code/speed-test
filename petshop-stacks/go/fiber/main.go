package main

import (
	"context"
	"log"
	"os"

	sw "github.com/GIT_USER_ID/GIT_REPO_ID/go"
)

func main() {
	ctx := context.Background()
	store, err := sw.NewPostgresStoreFromEnv(ctx)
	if err != nil {
		log.Fatalf("PostgreSQL connection failed: %v", err)
	}
	defer store.Close()

	routes := sw.ApiHandleFunctions{
		PetAPI:   sw.PetAPI{Store: store},
		StoreAPI: sw.StoreAPI{Store: store},
		UserAPI:  sw.UserAPI{Store: store},
	}

	app := sw.NewFiberApp(routes)
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Fatal(app.Listen(":" + port))
}
