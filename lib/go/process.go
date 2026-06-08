package main
import "fmt"
func process(v []int, base int) []int {
    n := []int{base}
    for _, x := range v {
        last := n[len(n)-1]
        if x == 1 { n = append(n, last+4) }
        } else if x == -1 { n = append(n, last-4) }
        } else { n = append(n, last) }
    }
    return n
}
func main() {
    v := []int{1,0,-1,1,0,-1,1,1}
    fmt.Println(process(v, 60))
}
