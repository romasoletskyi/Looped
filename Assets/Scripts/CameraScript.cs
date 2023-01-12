using UnityEngine;

public class CameraScript : MonoBehaviour
{
    [SerializeField] private GameObject _player;
    [Range(0f, 1f)] public float Smoothing = 0.1f;

    // Start is called before the first frame update
    void Start()
    {
        if (_player != null)
        {
            transform.position = _player.transform.position;
        }
    }

    // Update is called once per frame
    void FixedUpdate()
    {
        transform.position = Vector3.Lerp(transform.position, _player.transform.position, Smoothing * 50 * Time.fixedDeltaTime);
    }
}


