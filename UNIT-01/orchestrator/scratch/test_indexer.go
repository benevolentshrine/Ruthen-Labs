package scratch

import (
	"fmt"
	"unit01/clients"
)

func RunTestIndexer() {
	indexer := clients.NewIndexerClient()
	m, err := indexer.GetProjectMap("/Users/lichi/Ruthen-Labs/orchestrator")
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	} else {
		fmt.Printf("ProjectMap:\n%s\n", m)
	}
}
