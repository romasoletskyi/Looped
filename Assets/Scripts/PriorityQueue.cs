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
