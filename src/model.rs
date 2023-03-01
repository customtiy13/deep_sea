#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum NavigationalStatus {
    #[serde(rename = "Unknown value")]
    Unknown,
    #[serde(rename = "Under way using engine")]
    Under,
    #[serde(rename = "Engaged in fishing")]
    Engaged,
    #[serde(rename = "Moored")]
    Moored,
    #[serde(rename = "At anchor")]
    Anchor,
    #[serde(rename = "Restricted maneuverability")]
    Restricted,
    #[serde(rename = "Constrained by her draught")]
    Constrained,
    #[serde(rename = "Not under command")]
    Not,
    #[serde(rename = "Under way sailing")]
    Sailing,
    #[serde(rename = "Aground")]
    Aground,
    #[serde(alias = "Reserved for future use [11]")]
    #[serde(alias = "Reserved for future use [13]")]
    #[serde(alias = "Reserved for future amendment [HSC]")]
    Reserved,
    #[serde(other)]
    Other,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ShipType {
    Tanker,
    Cargo,
    Fishing,
    Passenger,
    Sailing,
    SAR,
    Pleasure,
    Reserved,
    Tug,
    WIG,
    Medical,
    #[serde(alias = "Anti-pollution")]
    AntiPollution,
    Other,
    #[serde(alias = "Towing long/wide")]
    Towing,
    Pilot,
    Dredging,
    Military,
    HSC,
    #[serde(alias = "Law enforcement")]
    Law,
    #[serde(alias = "Port tender")]
    Port,
    #[serde(alias = "Diving")]
    Diving,
    #[serde(alias = "Spare 1")]
    #[serde(alias = "Spare 2")]
    Spare,
    Undefined,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum DataSource {
    AIS,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
//1.	Timestamp			Timestamp from the AIS basestation, format: 31/12/2015 23:59:59
//2.	Type of mobile			Describes what type of target this message is received from (class A AIS Vessel, Class B AIS vessel, etc)
//3.	MMSI				MMSI number of vessel
//4.	Latitude			Latitude of message report (e.g. 57,8794)
//5.	Longitude			Longitude of message report (e.g. 17,9125)
//6.	Navigational status		Navigational status from AIS message if available, e.g.: 'Engaged in fishing', 'Under way using engine', mv.
//7.	ROT				Rot of turn from AIS message if available
//8.	SOG				Speed over ground from AIS message if available
//9.	COG				Course over ground from AIS message if available
//10.	Heading			Heading from AIS message if available
//11.	IMO				IMO number of the vessel
//12.	Callsign			Callsign of the vessel
//13.	Name				Name of the vessel
//14.	Ship type			Describes the AIS ship type of this vessel
//15.	Cargo type			Type of cargo from the AIS message
//16.	Width				Width of the vessel
//17.	Length				Lenght of the vessel
//18.	Type of position fixing device	Type of positional fixing device from the AIS message
//19.	Draught			Draugth field from AIS message
//20.	Destination			Destination from AIS message
//21.	ETA				Estimated Time of Arrival, if available
//22.	Data source type		Data source type, e.g. AIS
//23. Size A				Length from GPS to the bow
//24. Size B				Length from GPS to the stern
//25. Size C				Length from GPS to starboard side
//26. Size D				Length from GPS to port side
// example: 23/06/2017 00:00:00,Base Station,2194005,56.344267,4.272000,Unknown value,,,,,Unknown,,,Undefined,,,,Surveyed,,,,AIS,,,,
// 325315:23/06/2017 00:44:23,Class A,305484000,56.134323,11.474578,Under way using engine,0.0,6.8,268.1,264,9428217,V2EN3,ICE MOON,Cargo,,24,129,GPS,6.8,AARHUS,23/06/2017 08:00:00,AIS,109,20,12,12
pub struct Record {
    pub timestamp: String,
    type_mobile: String,
    pub mmsi: String,
    pub lat: f64,
    pub lon: f64,
    pub status: NavigationalStatus,
    rot: String,
    pub sog: Option<f64>,
    pub cog: Option<f64>,
    heading: String,
    imo: String,
    callsign: String,
    name: String,
    pub ship_type: ShipType,
    cargo_type: String,
    width: String,
    length: String,
    type_device: String,
    draught: String,
    destination: String,
    eta: String,
    pub data_source: DataSource,
    a: String,
    b: String,
    c: String,
    d: String,
}

#[derive(Debug)]
pub struct STPoint {
    pub timestamp: String,
    pub lat: f64,
    pub lon: f64,
    pub sog: f64,
    pub cog: f64,
}

#[derive(Debug)]
pub struct Trajectory {
    pub mmsi: String,
    pub ship_type: ShipType,
    pub trace: Vec<STPoint>,
}
