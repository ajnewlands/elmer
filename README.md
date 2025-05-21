Elmer is a utility inspired by my collection of python helper scripts that I use both in developing distributed applications based on RabbitMQ, and in debugging issues at customer sites. In particular, I wanted to start collecting the functionality of these scripts into something that would be portable (a single-file binary that runs on Windows/Mac/Linux) and had a suitable GUI that would make it immediately accessible for colleagues.

![image](https://github.com/user-attachments/assets/fae074ab-c3d2-4bb9-bdcf-5ffd838f07fa)

Upon first starting Elmer, and later upon clicking the 'connection settings' button ![image](https://github.com/user-attachments/assets/e6eb92fe-b205-428e-a0ea-92e4dd12c782)
, you will be confronted with the settings dialogue window:

![image](https://github.com/user-attachments/assets/804954c9-a5d5-4f69-97f8-531bada6cf4a)

Hopefully most of this is immediately familiar, and you know the correct settings for your instance.
The 'Wildcard subscription' option, if selected, will cause an initial subscription to be made with no qualifiers, i.e. to every message on the selected exchange. Hopefully this is what you want, because Elmer doesn't yet have dynamic subscription support (coming very soon!).

In the event that the accumulated data gets out of hand, you can clear it by clicking the 'clear data' button; ![image](https://github.com/user-attachments/assets/f02e8441-42f6-4f64-bb2a-ffdf855c6e04)

You can also search and filter by entering regular expressions in the filter entry box. Highlighted strings will be matched, and non-matching results will be hidden. By default only headers are searched, on the assumption that these containing message routing information, and message bodies are likely to be large indeed. However, you can check the 'filter body' option as well (under the hamburger menu) to search everything.


