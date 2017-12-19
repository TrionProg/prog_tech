use std;
use cgmath;

pub type Pos2D = cgmath::Point2<f32>;
pub type Pos3D = cgmath::Point3<f32>;

#[derive(Copy,Clone)]
pub struct Scale(pub f32);

pub type Deg = cgmath::Deg<f32>;
pub type Rad = cgmath::Rad<f32>;
//pub type Euler = cgmath::Euler<f32>;
pub type Quaternion = cgmath::Quaternion<f32>;

pub type Matrix4 = cgmath::Matrix4<f32>;

#[derive(Copy,Clone)]
pub struct Location {
    pub position:Pos3D,
    pub scale:Scale,
    pub rotation:Quaternion,
}

impl Location {
    pub fn new(position:Pos3D,scale:Scale,rotation:Quaternion) -> Self {
        Location{
            position:position,
            scale:scale,
            rotation:rotation,
        }
    }

    pub fn identity() -> Self {
        Location{
            position:Pos3D::new(0.0,0.0,0.0),
            scale:Scale(1.0),
            rotation:Quaternion::new(1.0,0.0,0.0,0.0),
        }
    }

    pub fn calculate_matrix(&self) -> Matrix4 {
        use cgmath::SquareMatrix;
        use cgmath::EuclideanSpace;

        Matrix4::from_translation(self.position.to_vec())*
        Matrix4::from(self.rotation)*
        Matrix4::from_scale(self.scale.0)
    }

    fn quat_from_matrix(mat:&[f32; 16]) -> Quaternion {
        use cgmath::InnerSpace;
        
        let t=mat[0] + mat[5] + mat[10] + 1.0;

        if t>0.0 {
            let s = 0.5 / t.sqrt();
            let w = 0.25 / s;
            let x = ( mat[9] - mat[6] ) * s;
            let y = ( mat[2] - mat[8] ) * s;
            let z = ( mat[4] - mat[1] ) * s;

            Quaternion::new(w, x, y, z).normalize()
        }else if mat[0]>mat[5] && mat[0]>mat[10] {
            let s = (( 1.0 + mat[0] - mat[5] - mat[10] ) * 2.0).sqrt();
            let x = 0.5 / s;
            let y = (mat[1] + mat[4] ) / s;
            let z = (mat[2] + mat[8] ) / s;
            let w = (mat[6] + mat[9] ) / s;

            Quaternion::new(w, x, y, z).normalize()
        }else if mat[5]>mat[0] && mat[5]>mat[10] {
            let s = (( 1.0 + mat[5] - mat[0] - mat[10] ) * 2.0).sqrt();
            let x = (mat[1] + mat[4] ) / s;
            let y = 0.5 / s;
            let z = (mat[6] + mat[9] ) / s;
            let w = (mat[2] + mat[8] ) / s;

            Quaternion::new(w, x, y, z).normalize()
        }else{
            let s = (( 1.0 + mat[10] - mat[0] - mat[5] ) * 2.0).sqrt();
            let x = (mat[2] + mat[8] ) / s;
            let y = (mat[6] + mat[9] ) / s;
            let z = 0.5 / s;
            let w = (mat[1] + mat[4] ) / s;

            Quaternion::new(w, x, y, z).normalize()
        }
    }

    fn scale_from_matrix(mat:&[f32; 16]) -> Result<Scale,()> {
        let scale_x = ((mat[0].powi(2) + mat[4].powi(2) + mat[8].powi(2)).sqrt()*100.0).round()/100.0;
        let scale_y = ((mat[1].powi(2) + mat[5].powi(2) + mat[9].powi(2)).sqrt()*100.0).round()/100.0;
        let scale_z = ((mat[2].powi(2) + mat[6].powi(2) + mat[10].powi(2)).sqrt()*100.0).round()/100.0;

        if scale_x != scale_y || scale_y != scale_z || scale_z != scale_x {
            return Err( () );
        }

        Ok( Scale(scale_x) )
    }

    pub fn from_matrix(matrix:&Matrix4) -> Result<Location,()> {
        use cgmath::Matrix;

        let standart_matrix=matrix.transpose();
        let mat:&[f32; 16]=standart_matrix.as_ref();

        let position=Pos3D::new(mat[3], mat[7], mat[11]);
        let scale=Self::scale_from_matrix(mat)?;
        let rotation=Self::quat_from_matrix(mat);

        Ok ( Location::new(position,scale,rotation) )
    }

}

impl PartialEq for Location {
    fn eq(&self, other:&Self) -> bool {
        const eps:f32 = 0.00001;

        let pos1=&self.position;
        let pos2=&other.position;

        if (pos1.x - pos2.x).abs() > eps || (pos1.y - pos2.y).abs() > eps || (pos1.z - pos2.z).abs() > eps {
            return false;
        }

        if (self.scale.0 - other.scale.0).abs() > eps {
            return false;
        }

        let rot1=&self.rotation;
        let rot2=&other.rotation;

        if (rot1.v.x - rot2.v.x).abs() > eps || (rot1.v.y - rot2.v.y).abs() > eps || (rot1.v.z - rot2.v.z).abs() > eps ||
            (rot1.s - rot2.s).abs() > eps
        {
            return false;
        }

        true
    }

    fn ne(&self, other:&Self) -> bool {
        !self.eq(other)
    }
}

impl std::ops::Sub for Location {
    type Output = Location;

    fn sub(self, other: Location) -> Location {
        use cgmath::EuclideanSpace;

        Location {
            position: Pos3D::from_vec(self.position-other.position),
            scale: Scale(self.scale.0/other.scale.0),
            rotation: self.rotation-other.rotation,
        }
    }
}

impl From<Matrix4> for Location {
    fn from(matrix:Matrix4) -> Self {
        /*
        let position = Position::with_asset(self.mat[3], self.mat[7], self.mat[11], asset);

        let scale_x = ((self.mat[0].powi(2) + self.mat[4].powi(2) + self.mat[8].powi(2)).sqrt()*100.0).round()/100.0;
        let scale_y = ((self.mat[1].powi(2) + self.mat[5].powi(2) + self.mat[9].powi(2)).sqrt()*100.0).round()/100.0;
        let scale_z = ((self.mat[2].powi(2) + self.mat[6].powi(2) + self.mat[10].powi(2)).sqrt()*100.0).round()/100.0;

        let scale = Scale::with_asset(scale_x, scale_y, scale_z, asset);

        let quat=self.to_quat(asset);

        Location::new(position, scale, quat)
        */

        Location::identity()//TODO:fix this
    }
}

/*
pub fn calculate_matrix(location:&Location) -> Matrix4 {
    use cgmath::SquareMatrix;
    use cgmath::Vector3;
    use cgmath::EuclideanSpace;
    //let mut matrix=Matrix4::identity();//from(location.rotation);
    let mut matrix=Matrix4::from_translation(location.position.to_vec())*Matrix4::from(location.rotation);
    /*
    matrix[3][0]=location.position.x;
    matrix[3][1]=location.position.y;
    matrix[3][2]=location.position.z;
    */

    //Matrix4::from_scale(3.0)*matrix//*Matrix4::from_scale(3.0)
    matrix
}
*/*/
