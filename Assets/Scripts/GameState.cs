using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public delegate void GameEvent();

public enum GamePeriod
{
    Running,
    Starting,
    Ending
}

public class GameState : MonoBehaviour
{
    public float RunTime { get; set; } = 30;
    public float GossipRange { get; set; } = 20; // distance direct gossips can reach
    public float GossipageLowerLimit { get; set; } = 0.3f; // number under which there is no gossip
    public float GossipageUpperLimit { get; set; } = 2f; // number on top of which everyone gossips

    static public GameState Main { get; private set; }

    public GamePeriod State { get; private set; } = GamePeriod.Starting;

    public List<NPCScript> NPCs { get; private set; } 

    private PriorityQueue<GameEvent> endEvents;
    private PriorityQueue<GameEvent> startEvents;

    private float _startTime;
    private float _deltaRunTime;

    public void Gossip (float ActionImportance, float Rebellion, float Scariness) // parameters ranged between 0 and 1, except scariness between -1 and 1
    {
        float gossipage = 0;
        int categories = 0;
        foreach (NPCScript script in NPCs) 
        { 
            if (Vector3.Distance(script.transform.position, this.transform.position) < GossipRange) 
            { 
                gossipage += script.Popularity;
                categories |= (int)script.Category;
            } 
        }
        gossipage *= ActionImportance;
        if (gossipage < GossipageLowerLimit) return;
        else if (gossipage > GossipageUpperLimit) { foreach (NPCScript script in NPCs) { script.RunGossip(ActionImportance, Rebellion, Scariness); } }
        else
        {
            foreach (NPCScript script in NPCs)
            {
                if (((int) script.Category & categories) != 0) script.RunGossip(ActionImportance, Rebellion, Scariness);
            }
        }
    }

    // Start is called before the first frame update
    private void Reset()
    {
        Main = this;
        NPCs = new List<NPCScript>();
        endEvents = new PriorityQueue<GameEvent>();
        startEvents = new PriorityQueue<GameEvent>();
    }

    public void AddNPC (NPCScript npc) { NPCs.Add(npc); }

    // Update is called once per frame
    private void Update()
    {
        if (State == GamePeriod.Running && Time.fixedTime - _startTime > _deltaRunTime)
        {
            PriorityQueue<GameEvent>.Node n = endEvents.Beginning;
            while (n != null) 
            {
                n.Object();
                n = n.Next;
            }
            StartCoroutine(Ending());
        }
        else if (State == GamePeriod.Starting)
        {
            PriorityQueue<GameEvent>.Node n = startEvents.Beginning;
            while (n != null)
            {
                n.Object();
                n = n.Next;
            }
            _startTime = Time.fixedTime;
            _deltaRunTime = RunTime;
            State = GamePeriod.Running;
        } 
    }

    private IEnumerator Ending()
    {
        yield return new WaitForSeconds(1f);
        State = GamePeriod.Starting;
        yield return null;
    }

    public void addStartEvent (GameEvent e, float priority) {startEvents.Insert(e, priority);} 
    public void addEndEvent(GameEvent e, float priority) { endEvents.Insert(e, priority); } 

}
