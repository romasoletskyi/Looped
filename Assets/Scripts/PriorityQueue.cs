using System.Collections.Generic;
using System.Linq;

public class PriorityQueue<T>
{
    public class Node
    {
        public Node(Node prev, Node next, float priority, T obj)
        {
            Next = next;
            Previous = prev;
            Priority = priority;
            Object = obj;
        }
        public Node Next { get; set; }
        public Node Previous { get; set; }
        public float Priority { get; private set; }
        public T Object { get; private set; }
    }

    public Node Beginning { get; private set; }

    public PriorityQueue () 
    {
        Beginning = null;
    }

    public void Insert (T obj, float priority)
    {
        Node prevNode = Beginning;
        if (prevNode == null)
        {
            Beginning = new Node(null, null, priority, obj);
            return;
        }
        while (prevNode.Priority > priority)
        {
            if (prevNode.Next == null)
            {
                Node node = new Node (prevNode, null, priority, obj);
                prevNode.Next = node;
                return;
            }
            prevNode = prevNode.Next;
        }
        if (prevNode.Previous == null)
        {
            Node node = new Node(null, prevNode, priority, obj);
            Beginning = node;
            prevNode.Previous = node;
            return;
        }
        Node n = new Node(prevNode.Previous, prevNode, priority, obj);
        prevNode.Previous.Next = n;
        prevNode.Previous = n;
        return;
    }
}

public class PriorityQueue<T, Priority> {
    class Element {
        public T value {get; set;}
        public Priority priority {get; set;}

        public Element(T value, Priority priority) {
            this.value = value;
            this.priority = priority;
        }
    }

    List<Element> heap;
    Dictionary<T, int> indices;

    public bool Empty() {
        return heap.Empty();
    }

    public void AddWithPriority(T value, Priority priority) {
        this.heap.Add(new Element(value, priority));
        this.indices.Add(value, this.heap.Count - 1);
        this.HeapifyUp(this.heap.Count - 1);
    }

    public T ExctractMin() {
        (this.heap[0], this.heap.Last()) = (this.heap.Last(), this.heap[0]);
        T value = this.heap.Last().value;

        this.heap.RemoveAt(this.heap.Count - 1);
        this.indices.Remove(value);
        this.HeapifyDown(0);

        return value;
    }

    public void DecreasePriority(T value, Priority new_priority) {
        int index = this.indices[value];
        this.heap[index].priority = new_priority;
        HeapifyUp(index);
    }

    private void HeapifyUp(int index) {
        int parent = (index - 1) / 2;

        if (this.heap[index].priority < this.heap[parent].priority) {
            (this.heap[index], this.heap[parent]) = (this.heap[parent], this.heap[index]);
            this.HeapifyUp(parent);
        }
    }

    private void HeapifyDown(int index) {
        int left = 2 * index + 1;
        int right = 2 * index + 2;
        int smallest = index;

        if (left < this.heap.Count && this.heap[left].priority < this.heap[smallest].priority) {
            smallest = left;
        }

        if (right < this.heap.Count && this.heap[right].priority < this.heap[smallest].priority) {
            smallest = right;
        }

        if (smallest != index) {
            (this.heap[index], this.heap[smallest]) = (this.heap[smallest], this.heap[index]);
            this.HeapifyDown(smallest);
        }
    }
}
